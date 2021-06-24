use crate::random_hash_set::RandomHashSet;
use super::connection::Connection;
use std::cmp::Ordering;

pub struct Node {
    pub(super) connections: RandomHashSet<Connection>,
    pub(super) output: Option<f64>,
    pub x: f64,
}

impl Node {
    pub fn new(x: f64) -> Self {
        Node {connections: RandomHashSet::new(), output: None, x}
    }

    // goes over connections and processes from_node's output with connection weight,
    // averages this, then yeets through activation function
    pub fn run_node<F>(&mut self, activation_function: F) where
        F: Fn(f64) -> f64 {
        self.output = Some(self.get_output(activation_function)); // gets output, saves
    }

    pub fn get_output<F>(&self, activation_function: F) -> f64 where
        F: Fn(f64) -> f64 {
        let mut total_in: f64 = 0.0;

        for connection_rc in self.connections.get_data() {
            let node_from = &connection_rc.node.borrow();

            if let None = node_from.output {
                panic!("ERROR, no output in input node - illegal for this basic feed-forward network");
            }

            let connection_weight = connection_rc.weight;
            let value = node_from.output.unwrap() * connection_weight;

            total_in += value; // add weighted output to total_ins
        }

        let pre_activated_output = total_in;
        return activation_function(pre_activated_output); // return activated output
    }

    //for setting the input nodes' values
    pub fn set_output(&mut self, output: Option<f64>) {
        self.output = output;
    }
}

impl PartialOrd for Node {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.x > other.x {
            Some(Ordering::Greater)
        }
        else if self.x < other.x {
            Some(Ordering::Less)
        }
        else {
            Some(Ordering::Equal)
        }
    }
}

impl Ord for Node {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.x > other.x {
            Ordering::Greater
        }
        else if self.x < other.x {
            Ordering::Less
        }
        else {
            Ordering::Equal
        }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.connections.get_data() == other.connections.get_data() && self.output == other.output && self.x == other.x
    }
}

impl Eq for Node {}