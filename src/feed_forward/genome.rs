use super::node_gene::NodeGene;
use super::connection_gene::ConnectionGene;
use super::gene::Gene;
use std::rc::Rc;
use std::collections::HashMap;

pub struct Genome {
    pub(crate) connections: HashMap<usize, ConnectionGene>, //connections with same inv_num need to share to/from nodes
    pub(crate) nodes: HashMap<usize, Rc<NodeGene>>, //nodes with same inv_num need to be exact same node (to maintain x,y values) - not Rc as a node's values are constant
}

impl Genome {
    pub fn new() -> Self {
        Genome {connections: HashMap::new(), nodes: HashMap::new()}
    }

    //If node not already contained, add
    pub fn add_node(&mut self, node: Rc<NodeGene>) {
        if !self.nodes.contains_key(&node.get_innovation_number()) {
            self.nodes.insert(node.get_innovation_number(), node);
        }
    }

    pub fn add_connection(&mut self, connection: ConnectionGene) {
        if !self.connections.contains_key(&connection.get_innovation_number()) {
            self.connections.insert(connection.get_innovation_number(), connection);
        }
    }
}

impl PartialEq for Genome {
    fn eq(&self, other: &Self) -> bool {
        self.nodes.len() == other.nodes.len()
            && self.connections.len() == other.connections.len()
            && self.nodes == other.nodes
            && self.connections == other.connections
    }
}

impl Eq for Genome {}