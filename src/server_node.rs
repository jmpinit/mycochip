use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, mpsc, Mutex};
use std::sync::mpsc::{Receiver, Sender, TryRecvError};
use std::thread;

type ClientMap = Arc<Mutex<HashMap<u16, Client>>>;

fn handle_client(mut stream: TcpStream, client_id: u16, clients: ClientMap) {
    let (request_tx, request_rx) = mpsc::channel::<Vec<u8>>();

    let mut stream_for_rx = stream;//Arc::new(Mutex::new(stream));
    let mut stream_for_tx = stream_for_rx.try_clone().unwrap();

    thread::spawn(move || {
        let mut buf = [0; 1024];

        loop {
            let len = stream_for_rx.read(&mut buf).unwrap();

            if len == 0 {
                break;
            }

            request_tx.send(buf[..len].to_vec()).unwrap();
        }
    });

    clients.lock().unwrap().insert(client_id, Client::new());

    loop {
        match request_rx.try_recv() {
            Ok(buf) => {
                let mut locked_clients = clients.lock().unwrap();
                let client = locked_clients.get_mut(&client_id).unwrap();
                client.rx_buffer.extend_from_slice(&buf);
            }
            Err(TryRecvError::Empty) => {},
            Err(TryRecvError::Disconnected) => {
                clients.lock().unwrap().remove(&client_id);
                println!("Client {} disconnected", client_id);
                break;
            },
        }

        let mut locked_clients = clients.lock().unwrap();
        // let client = locked_clients.get_mut(&client_id).unwrap();

        match locked_clients.get_mut(&client_id) {
            Some(client) => {
                let buffer = &mut client.tx_buffer;

                if buffer.len() > 0 {
                    match stream_for_tx.write(&buffer) {
                        Ok(bytes_written) => {
                            buffer.drain(0..bytes_written);
                        }
                        Err(_) => {
                            println!("Cannot write data, client {} disconnected", client_id);
                        }
                    }
                }
            }
            None => {
                println!("Client {} disconnected", client_id);
                break;
            }
        }
    }
}

struct Client {
    rx_buffer: Vec<u8>,
    tx_buffer: Vec<u8>,
}

impl Client {
    pub fn new() -> Self {
        Self {
            rx_buffer: Vec::new(),
            tx_buffer: Vec::new(),
        }
    }
}

pub struct ServerNode {
    clients: ClientMap,
}

impl ServerNode {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(&self, address: &str) {
        let listener = TcpListener::bind(address).unwrap();
        let mut next_client_id: u16 = 0;

        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let client_data = self.clients.clone();
                    let client_id = next_client_id;
                    next_client_id += 1;

                    thread::spawn(move || handle_client(stream, client_id, client_data));
                }
                Err(_) => {
                    // Handle error
                }
            }
        }
    }

    pub fn connected_client_ids(&self) -> Vec<u16> {
        let data = self.clients.lock().unwrap();
        data.keys().cloned().collect()
    }

    pub fn read_data(&self, client_id: u16) -> Option<Vec<u8>> {
        match self.clients.lock() {
            Ok(mut client_map) => {
                match client_map.get_mut(&client_id) {
                    Some(client) => {
                        let buffer = &mut client.rx_buffer;

                        if buffer.len() == 0 {
                            return None;
                        }

                        let data = buffer.clone();
                        buffer.clear();
                        return Some(data);
                    }
                    None => None
                }
            }
            Err(_) => None
        }
    }

    pub fn send_data(&self, client_id: u16, data: &Vec<u8>) {
        if let Some(client) = self.clients.lock().unwrap().get_mut(&client_id) {
            let buffer = &mut client.tx_buffer;
            buffer.extend(data);
        }
    }

    pub fn is_connected(&self, client_id: u16) -> bool {
        let data = self.clients.lock().unwrap();
        data.contains_key(&client_id)
    }

    pub fn disconnect(&self, client_id: u16) {
        self.clients.lock().unwrap().remove(&client_id);
    }

    pub fn disconnect_all(&self) {
        self.clients.lock().unwrap().clear();
    }
}
