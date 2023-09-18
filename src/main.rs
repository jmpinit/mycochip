use std::time::Instant;
use std::{io, thread};
use std::cell::RefCell;
use std::io::{Write};
use std::collections::{HashMap};
use std::fmt::format;
use std::rc::Rc;
use std::sync::Arc;
use crate::avr_net::AvrNetMessage;
use crate::config::MycochipConfig;
use crate::network::NetworkReceive;
use crate::server_node::ServerNode;

mod cli;
mod config;
mod comms;
pub mod avr_simulator;
mod network;
mod server_node;
mod avr_net;

type AvrSimulatorRef = Rc<RefCell<avr_simulator::AvrSimulator>>;

const REQUEST_PORT: i32 = 6711;
const DEVICE_EVENT_PORT: i32 = 6712;
const TCP_GATEWAY_NAME: &str = "tcp_gateway";
const TCP_GATEWAY_ADDRESS: u16 = 1;

fn cmd_rx(channel_name: &str) {
    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    subscriber.set_subscribe(channel_name.as_bytes()).unwrap();

    let listen_address = format!("tcp://localhost:{}", DEVICE_EVENT_PORT);
    assert!(subscriber.connect(listen_address.as_str()).is_ok());

    loop {
        let mut msg = zmq::Message::new();
        subscriber.recv(&mut msg, 0).unwrap(); // Clear the topic name
        subscriber.recv(&mut msg, 0).unwrap();

        let msg_bytes = &msg as &[u8];
        let msg_str = std::str::from_utf8(msg_bytes).unwrap();
        let msg_printable = msg_str.replace(|c: char| !c.is_ascii(), "");

        print!("{}", msg_printable);
        io::stdout().flush().unwrap();
    }
}

fn cmd_list() {
    let req = comms::request::Request {
        command_type: comms::request::CommandType::List.into(),
        args: None,
    };

    let res = comms::send_request(&req).unwrap();

    println!("Received: {}", res.as_str().unwrap());
}

fn cmd_pin(machine_name: &str, port: &str, pin_index: &u8, state: Option<&bool>) {
    let req = comms::request::Request {
        command_type: comms::request::CommandType::Io.into(),
        args: Some(comms::request::request::Args::IoArgs(comms::request::IoArgs {
            machine_id: machine_name.to_string(),
            port: port.to_string(),
            pin_index: *pin_index as u32,
        })),
    };

    let res = comms::send_request(&req).unwrap();

    println!("Received: {}", res.as_str().unwrap());
}

struct AvrReceiver {
    avr: AvrSimulatorRef,
}

impl<'a> NetworkReceive<'a> for AvrReceiver {
    fn receive(&mut self, b: u8) {
        self.avr.borrow_mut().write_uart('0', b);
    }
}

struct TcpReceiver {
    // Parses messages from the chips in the network
    avr_net_node: avr_net::AvrNetState,
    tcp_server: Arc<ServerNode>,
}

impl TcpReceiver {
    fn new(tcp_server: Arc<ServerNode>) -> Self {
        Self {
            avr_net_node: avr_net::AvrNetState::new(TCP_GATEWAY_ADDRESS),
            tcp_server,
        }
    }
}

impl NetworkReceive<'_> for TcpReceiver {
    fn receive(&mut self, b: u8) {
        let data = vec![b];
        for id in self.tcp_server.connected_client_ids() {
            self.tcp_server.send_data(id, &data);
        }
    }
}

fn init_network(network: &mut network::Network, devs: &mut HashMap<String, AvrSimulatorRef>, config: &MycochipConfig) {
    for (device_name, device) in &config.devices {
        let eeprom = match &device.eeprom {
            Some(eeprom) => Some(eeprom.as_slice()),
            None => None,
        };

        let avr = Rc::new(RefCell::new(avr_simulator::AvrSimulator::new(
            &device.mcu,
            u32::MAX,
            &device.firmware,
            eeprom,
        )));

        let avr_receiver = AvrReceiver { avr: avr.clone() };
        network.create_node(device_name, avr_receiver);

        devs.insert(device_name.clone(), avr);

        println!("Started a {0} named {1}", device.mcu, device_name);
    }

    // Connect the network
    for (device_name, device) in &config.devices {
        for peer_name in &device.peers {
            network.connect(device_name, peer_name);
        }
    }
}

fn publish_bus_data(node_name: &str, publisher: &zmq::Socket, data: &Vec<u8>) -> Result<(), zmq::Error> {
    let topic = format!("{}/bus", node_name);
    publisher.send(topic.as_bytes(), zmq::SNDMORE)?;
    publisher.send(data, 0)
}

fn publish_pin_event(node_name: &str, publisher: &zmq::Socket, port: char, pin_index: u8, state: bool) -> Result<(), zmq::Error> {
    let topic = format!("{}/pin/{}/{}", node_name, port, pin_index);
    publisher.send(topic.as_bytes(), zmq::SNDMORE)?;
    publisher.send( (state as u8).to_string().as_bytes(), 0)
}

struct PinTracker {
    last_port_values: HashMap<char, u8>,
}

impl PinTracker {
    fn new() -> Self {
        Self {
            // TODO: allow for parts with different number of ports
            last_port_values: HashMap::from([
                ('B', 0),
                ('C', 0),
                ('D', 0),
            ]),
        }
    }

    fn update(&mut self, avr: &mut avr_simulator::AvrSimulator) -> Vec<(char, u8, bool)> {
        let mut pin_events: Vec<(char, u8, bool)> = vec![];
        let mut new_values: HashMap<char, u8> = HashMap::new();

        for (port_name, last_port_value) in self.last_port_values.iter_mut() {
            // TODO: retrieve the whole port at once
            let mut port_value = 0;

            for bit_idx in 0..8 {
                let last_state = ((*last_port_value >> bit_idx) & 1) > 0;
                let current_state = avr.get_digital_pin(*port_name, bit_idx);

                if last_state != current_state {
                    pin_events.push((*port_name, bit_idx, current_state));
                }

                port_value |= (current_state as u8) << bit_idx;
            }

            new_values.insert(*port_name, port_value);
        }

        self.last_port_values = new_values;

        pin_events
    }
}

fn cmd_up(config_file_path: &str) {
    let config_or_err = config::load(config_file_path);

    if config_or_err.is_err() {
        println!("Error: {}", config_or_err.err().unwrap());
        return;
    }

    let config = config::load(config_file_path).unwrap();

    let mut devs: HashMap<String, AvrSimulatorRef> = HashMap::new();
    let mut pin_trackers: HashMap<String, PinTracker> = HashMap::new();
    let mut network = network::Network::new();

    // ZMQ sockets
    let context = zmq::Context::new();
    let responder = context.socket(zmq::REP).unwrap();
    responder.set_linger(0).unwrap();
    let publisher = context.socket(zmq::PUB).unwrap();
    publisher.set_linger(0).unwrap();

    {
        let responder_address = format!("tcp://*:{}", REQUEST_PORT);
        assert!(responder.bind(responder_address.as_str()).is_ok());
        let pub_address = format!("tcp://*:{}", DEVICE_EVENT_PORT);
        println!("Publishing on {}", pub_address);
        assert!(publisher.bind(pub_address.as_str()).is_ok());
    }

    // TCP server
    let tcp_server_for_rx = Arc::new(ServerNode::new("0.0.0.0:7001"));
    let tcp_server_for_tx = tcp_server_for_rx.clone();
    {
        let tcp_server = tcp_server_for_rx.clone();
        thread::spawn(move || tcp_server.start());
    }

    {
        let tcp_server = tcp_server_for_rx.clone();
        ctrlc::set_handler(move || {
            println!("Shutting down");
            tcp_server.shutdown();
            std::process::exit(0);
        })
        .expect("Error setting Ctrl-C handler");
    }

    let tcp_receiver = TcpReceiver::new(tcp_server_for_tx);
    network.create_node(TCP_GATEWAY_NAME, tcp_receiver);
    init_network(&mut network, &mut devs, &config);

    for (device_name, _) in &devs {
        pin_trackers.insert(device_name.clone(), PinTracker::new());
    }

    let mut msg = zmq::Message::new();
    loop {
        let now = Instant::now();

        // Collect messages sent from the devices
        for (device_name, dev) in devs.iter_mut() {
            let data: Vec<u8> = std::iter::from_fn(|| dev.borrow_mut().read_uart('0')).collect();

            if data.len() == 0 {
                continue;
            }

            network.broadcast_from(device_name, &data);

            // Publish for external listeners
            publish_bus_data(device_name, &publisher, &data).unwrap();
        }

        // Collect messages sent from the world to the devices
        let client_ids = tcp_server_for_rx.connected_client_ids();
        for id in client_ids {
            if let Some(buf) = tcp_server_for_rx.read_data(id) {
                let tcp_data = buf;

                let avr_net_message = Vec::try_from(AvrNetMessage {
                    address: 1, // HACK: hard-coded address
                    data: tcp_data.clone(),
                }).unwrap();

                println!("Sending message of size {} to AVR", avr_net_message.len());

                network.broadcast_from(TCP_GATEWAY_NAME, &avr_net_message);
                println!("Received: {}", String::from_utf8_lossy(&tcp_data));
            }
        }

        // Deliver queued messages
        network.deliver_messages();

        // Update the AVRs
        for _ in 1..1000 {
            for (_, dev) in devs.iter() {
                let _state = dev.borrow_mut().step();

                if _state.state == avr_simulator::state::AvrState::Sleeping {
                    break;
                }
            }
        }

        // Broadcast pin events
        for (node_name, dev) in devs.iter_mut() {
            let sim = &mut *dev.borrow_mut();
            let pin_tracker = pin_trackers.get_mut(node_name).unwrap();
            let pin_events = pin_tracker.update(sim);

            for (port, pin_index, state) in pin_events {
                publish_pin_event(node_name, &publisher, port, pin_index, state).unwrap();
            }
        }

        // Respond to requests

        if let Ok(_) = responder.recv(&mut msg, zmq::DONTWAIT) {
            let msg_bytes = &msg as &[u8];
            let req = comms::deserialize_request(msg_bytes).unwrap();

            match comms::request::CommandType::from_i32(req.command_type) {
                Some(comms::request::CommandType::List) => {
                    responder.send("device1, device2", 0).unwrap();
                },
                Some(comms::request::CommandType::Logs) => {
                    responder.send("hello from the logs", 0).unwrap();
                },
                Some(comms::request::CommandType::Io) => {
                    match req.args {
                        Some(comms::request::request::Args::IoArgs(ref io_args)) => {
                            let port_char: char = io_args.port.chars().next().unwrap();
                            let pin_state = devs.get(&io_args.machine_id)
                                .unwrap().borrow_mut()
                                .get_digital_pin(port_char, io_args.pin_index as u8);
                            let pin_msg = format!("{}", pin_state);
                            responder.send(pin_msg.as_str(), 0).unwrap();
                        },
                        _ => {
                            responder.send("Error!", 0).unwrap();
                        },
                    }
                }
                _ => {
                    responder.send("Error!", 0).unwrap();
                },
            }
        }

        // let elapsed = now.elapsed();
        // if elapsed > std::time::Duration::from_millis(10) {
        //     println!("Elapsed: {:.2?}", elapsed);
        // }
    }
}

fn main() {
    let matches = cli::build_cli().get_matches();

    match matches.subcommand() {
        Some(("up", args)) => {
            let config_file_path = match args.get_one::<String>("config-file") {
                Some(config_file_path) => config_file_path,
                None => "mycochip.yaml",
            };

            cmd_up(config_file_path);
        },
        Some(("list", _)) => cmd_list(),
        Some(("pin", args)) => {
            let node_name = args.get_one::<String>("node")
                .expect("Node name is required");
            let port = args.get_one::<String>("port")
                .expect("Port name is required");
            let pin_index = args.get_one::<u8>("pin")
                .expect("Pin number is required");
            let state = args.get_one::<bool>("state");

            cmd_pin(node_name, port, pin_index, state);
        },
        Some(("rx", args)) => {
            let node_name = args.get_one::<String>("node")
                .expect("Node name is required");

            cmd_rx(node_name);
        },
        _ => println!("No subcommand"),
    }
}
