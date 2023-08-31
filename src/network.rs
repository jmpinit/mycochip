use std::collections::{HashMap, VecDeque};

struct NetworkNode {
    // The names of the channels this node is connected to
    channel_names: Vec<String>,
}

pub struct Network {
    // The nodes in the network
    // node name -> node
    nodes: HashMap<String, NetworkNode>,
    // The messages queued to send since the last time we delivered messages
    // node name -> data queue
    to_send: HashMap<String, VecDeque<u8>>,
}

impl Network {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            to_send: HashMap::new(),
        }
    }

    pub fn add_node(&mut self, name: String, channel_names: Vec<String>) {
        self.nodes.insert(name.clone(), NetworkNode {
            channel_names,
        });
    }

    pub fn node_names(&self) -> Vec<String> {
        self.nodes.keys().map(|s| s.clone()).collect()
    }

    pub fn connect(&mut self, name: String, channel_name: String) {
        self.nodes
            .get_mut(&name).unwrap()
            .channel_names
            .push(channel_name);
    }

    pub fn broadcast_on(&mut self, sender_name: &String, channel_name: String, data: Vec<u8>) {
        // Add the data to the queue for each node that is connected to the channel
        for (node_name, node) in self.nodes.iter() {
            if node_name == sender_name {
                continue;
            }

            if node.channel_names.contains(&channel_name) {
                self.to_send
                    .entry(node_name.clone())
                    .or_insert_with(|| VecDeque::new())
                    .extend(data.iter());
            }
        }
    }

    // Broadcast a message from a node to all the other nodes on the same channel
    pub fn broadcast_from(&mut self, node_name: &String, data: Vec<u8>) {
        let channel_names: Vec<String> = self.nodes[node_name].channel_names.clone();

        for channel_name in channel_names {
            self.broadcast_on(node_name, channel_name.clone(), data.clone());
        }
    }

    // Read the messages queued for a node and remove them from the queue
    pub fn messages_for(&mut self, name: &String) -> Option<Vec<u8>> {
        // Get the data to send to this node
        match self.to_send.get_mut(name) {
            Some(data) => {
                if data.len() == 0 {
                    return None;
                }

                let mut data_copy = Vec::new();
                data_copy.extend(data.iter());
                data.clear();

                Some(data_copy)
            },
            None => None
        }
    }
}
