use std::time::Instant;
use std::{io, thread};
use std::io::{Write};
use std::collections::{HashMap};
use std::sync::Arc;
use crate::server_node::ServerNode;

mod cli;
mod config;
mod comms;
pub mod avr_simulator;
mod network;
mod server_node;
mod avr_net;

fn cmd_rx(channel_name: &str) {
    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    subscriber.set_subscribe(channel_name.as_bytes()).unwrap();

    assert!(subscriber.connect("tcp://localhost:6724").is_ok());

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

fn cmd_up(config_file_path: &str) {
    let config = config::load(config_file_path).unwrap();

    let mut devs: HashMap<String, avr_simulator::AvrSimulator> = HashMap::new();
    let mut network = network::Network::new();
    network.add_node("gateway".to_string(), vec!["tcp".to_string()]);

    for (device_name, device) in &config.devices {
        let avr = avr_simulator::AvrSimulator::new(
            &device.mcu,
            u32::MAX,
            &device.firmware,
        );

        network.add_node(device_name.clone(), device.channels.clone());

        devs.insert(device_name.clone(), avr);

        println!("Started a {0} named {1}", device.mcu, device_name);
    }

    let context = zmq::Context::new();
    let responder = context.socket(zmq::REP).unwrap();
    let publisher = context.socket(zmq::PUB).unwrap();

    // TCP node
    let mut tcp_avr_net_node = avr_net::AvrNetState::new(0);
    let tcp_server = Arc::new(ServerNode::new());
    let tcp_server_clone = tcp_server.clone();
    thread::spawn(move || tcp_server_clone.start("0.0.0.0:8000"));

    assert!(responder.bind("tcp://*:6723").is_ok());
    assert!(publisher.bind("tcp://*:6724").is_ok());

    let mut msg = zmq::Message::new();
    loop {
        let now = Instant::now();

        // Update the AVRs
        for _ in 1..1000 {
            for (_, dev) in devs.iter_mut() {
                let _state = dev.step();

                if _state.state == avr_simulator::state::AvrState::Sleeping {
                    break;
                }
            }
        }

        // Collect messages sent from the devices to the network
        for (device_name, dev) in devs.iter_mut() {
            let data: Vec<u8> = std::iter::from_fn(|| dev.read_uart('0')).collect();

            // HACK!
            // let eot: u8 = 4;
            // if data.contains(&eot) {
            //     println!("END OF TRANSMISSION");
            //     tcp_server.disconnect_all();
            // }

            if data.len() > 0 {
                for channel_name in &config.devices[device_name].channels {
                    publisher.send(channel_name.as_bytes(), zmq::SNDMORE).unwrap();
                    publisher.send(data.clone(), 0).unwrap();
                }

                // println!("Broadcasting from {} to {:?}", device_name, config.devices[device_name].channels);
                network.broadcast_from(device_name, data);
            }
        }

        // Collect messages sent from the network to the devices
        let client_ids = tcp_server.connected_client_ids();
        for id in client_ids {
            if let Some(buf) = tcp_server.read_data(id) {
                // HACK: minimize HTTP
                let tcp_data = if buf.starts_with("GET ".as_bytes()) {
                    let buf_str = String::from_utf8_lossy(&buf);
                    let first_line = buf_str.split("\r\n").next().unwrap().to_string();
                    first_line.as_bytes().to_vec()
                } else {
                    buf
                };

                let avr_net_message = {
                    let mut msg_data: Vec<u8> = Vec::new();

                    // Address
                    msg_data.push(0);
                    msg_data.push(1); // HACK: hard-coded address

                    // Length
                    let len: u16 = tcp_data.len() as u16;
                    msg_data.push((len >> 8) as u8);
                    msg_data.push(len as u8);

                    // Data
                    msg_data.extend(tcp_data.iter());

                    msg_data
                };

                println!("Sending message of size {} to AVR", avr_net_message.len());

                network.broadcast_on(&"gateway".to_string(), "tcp".to_string(), avr_net_message);
                println!("Received: {}", String::from_utf8_lossy(&tcp_data));
            }
        }

        // Deliver queued messages
        for device_name in network.node_names() {
            match network.messages_for(&device_name) {
                Some(data) => {
                    // HACK: need to implement a general purpose peripheral node type
                    // instead of hard-coding this here
                    if device_name == "gateway" {
                        // println!("Received message from TCP: {:?}", data);
                        for b in data {
                            if let Some(message_data) = tcp_avr_net_node.rx(b) {
                                println!("Received message from AVR: {:?}", message_data);
                                for id in tcp_server.connected_client_ids() {
                                    tcp_server.send_data(id, &message_data.to_vec());
                                }
                            }
                        }

                        continue;
                    }

                    // Regular message

                    let dev = devs.get_mut(&device_name).unwrap();

                    // println!("Delivering \"{}\" to {}", String::from_utf8_lossy(&data), device_name);
                    println!("Delivering message of size {} to {}", data.len(), device_name);

                    for b in data {
                        dev.write_uart('0', b);
                    }
                }
                None => {}
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
        Some(("rx", args)) => {
            let channel_name = args.get_one::<String>("channel")
                .expect("Channel name is required");

            cmd_rx(channel_name);
        },
        _ => println!("No subcommand"),
    }
}
