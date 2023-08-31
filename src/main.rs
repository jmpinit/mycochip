use std::{io, thread};
use std::io::{Write};
use std::collections::{HashMap};
use std::sync::Arc;
use crate::server_node::ServerNode;

mod cli;
mod config;
mod comms;
mod avr_simulator;
mod network;
mod server_node;

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
            1000000,
            &device.firmware,
        );

        network.add_node(device_name.clone(), device.channels.clone());

        devs.insert(device_name.clone(), avr);

        println!("Started a {0} named {1}", device.mcu, device_name);
    }

    let context = zmq::Context::new();
    let responder = context.socket(zmq::REP).unwrap();
    let publisher = context.socket(zmq::PUB).unwrap();
    // let (web_request_rx, web_reply_tx) = spawn_socket_in_channel();
    let tcp_server = Arc::new(ServerNode::new());
    let tcp_server_clone = tcp_server.clone();
    thread::spawn(move || tcp_server_clone.start("0.0.0.0:8000"));

    assert!(responder.bind("tcp://*:6723").is_ok());
    assert!(publisher.bind("tcp://*:6724").is_ok());

    let mut msg = zmq::Message::new();
    loop {
        // Update the AVRs
        for (_, dev) in devs.iter_mut() {
            let _state = dev.step();
        }

        // Collect messages sent from the devices to the network
        for (device_name, dev) in devs.iter_mut() {
            let data: Vec<u8> = std::iter::from_fn(|| dev.read_uart('0')).collect();

            // HACK!
            let eot: u8 = 4;
            if data.contains(&eot) {
                println!("END OF TRANSMISSION");
                tcp_server.disconnect_all();
            }

            if data.len() > 0 {
                for channel_name in &config.devices[device_name].channels {
                    publisher.send(channel_name.as_bytes(), zmq::SNDMORE).unwrap();
                    publisher.send(data.clone(), 0).unwrap();
                }

                // println!("Broadcasting from {} to {:?}", device_name, config.devices[device_name].channels);
                network.broadcast_from(device_name, data);
            }
        }

        let client_ids = tcp_server.connected_client_ids();
        for id in client_ids {
            if let Some(buf) = tcp_server.read_data(id) {
                network.broadcast_on(&"gateway".to_string(), "tcp".to_string(), buf.clone());
                println!("Received: {}", String::from_utf8_lossy(&buf));
            }
        }

        // Deliver queued messages
        for device_name in network.node_names() {
            match network.messages_for(&device_name) {
                Some(data) => {
                    // HACK: need to implement a general purpose peripheral node type
                    // instead of hard-coding this here
                    if device_name == "gateway" {
                        for id in tcp_server.connected_client_ids() {
                            tcp_server.send_data(id, &data);
                        }
                        continue;
                    }

                    // Regular message

                    let dev = devs.get_mut(&device_name).unwrap();

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

        // println!("Tick main loop");
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
