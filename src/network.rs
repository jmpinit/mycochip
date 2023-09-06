use std::collections::{HashMap, HashSet};

struct NetworkNode<'a> {
    name: String,
    // The names of the nodes this node is connected to
    peers: HashSet<String>,
    outgoing: Vec<u8>,
    receiver: Box<dyn NetworkReceive<'a> + 'a>,
}

pub trait NetworkReceive<'a>: 'a {
    fn receive(&mut self, b: u8);
}

impl<'a> NetworkNode<'a> {
    pub fn new(name: &str, receiver: impl NetworkReceive<'a>) -> Self {
        Self {
            name: name.to_string(),
            peers: HashSet::new(),
            outgoing: Vec::new(),
            receiver: Box::new(receiver),
        }
    }

    pub fn connect(&mut self, peer_name: &str) {
        if self.name == peer_name {
            panic!("Cannot connect a node to itself");
        }

        self.peers.insert(peer_name.to_string());
    }

    pub fn broadcast(&mut self) -> Vec<u8> {
        let mut data = Vec::new();
        data.extend(self.outgoing.iter());

        data
    }
}

pub struct Network<'a> {
    // The nodes in the network
    // node name -> node
    nodes: HashMap<String, NetworkNode<'a>>,
}

impl<'a> Network<'a> {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn create_node(&mut self, name: &str, receiver: impl NetworkReceive<'a>) {
        let new_node = NetworkNode::new(name, receiver);
        self.nodes.insert(name.to_string(), new_node);
    }

    pub fn destroy_node(&mut self, name: &str) {
        self.nodes.remove(name);
    }

    // Connect two nodes together bidirectionally
    pub fn connect(&mut self, node1_name: &str, node2_name: &str) {
        let node1 = self.nodes.get_mut(node1_name).unwrap();
        node1.connect(node2_name);

        let node2 = self.nodes.get_mut(node2_name).unwrap();
        node2.connect(node1_name);
    }

    pub fn disconnect(&mut self, node1_name: &str, node2_name: &str) {
        let node1 = self.nodes.get_mut(node1_name).unwrap();
        node1.peers.retain(|n| n != &node2_name);

        let node2 = self.nodes.get_mut(node2_name).unwrap();
        node2.peers.retain(|n| n != &node1_name);
    }

    // Broadcast a message from a node to all of its peers
    pub fn broadcast_from(&mut self, node_name: &str, data: &Vec<u8>) {
        let node = self.nodes.get_mut(node_name).unwrap();
        node.outgoing.extend(data.iter());
    }

    pub fn deliver_messages(&mut self) {
        let node_names = self.node_names();

        // Process messages
        for node_name in &node_names {
            // TODO: how do I avoid the clones here?
            let node = self.nodes.get(node_name).unwrap();
            let peer_names = node.peers.clone();
            let outgoing = node.outgoing.clone();

            for peer_name in peer_names {
                let peer = self.nodes.get_mut(peer_name.as_str()).unwrap();

                for b in &outgoing {
                    peer.receiver.receive(*b);
                }
            }
        }

        // Clear buffers
        for node_name in &node_names {
            let node = self.nodes.get_mut(node_name).unwrap();
            node.outgoing.clear();
        }
    }

    pub fn node_names(&self) -> Vec<String> {
        self.nodes.keys().map(|s| s.clone()).collect()
    }
}
