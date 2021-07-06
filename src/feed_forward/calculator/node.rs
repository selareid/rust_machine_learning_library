use crate::random_hash_set::RandomHashSet;
use super::connection::Connection;
use std::cmp::Ordering;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Node {
    pub(super) connections: RandomHashSet<Connection>,
    pub(super) output: Option<f64>,
    pub x: f64,
}

impl Node {
    pub(super) fn new(x: f64) -> Self {
        Node {connections: RandomHashSet::new(), output: None, x}
    }

    // goes over connections and processes from_node's output with connection weight,
    // averages this, then yeets through activation function
    pub(super) fn run_node<F>(&mut self, activation_function: F) where
        F: Fn(f64) -> f64 {
        self.output = Some(self.get_output(activation_function)); // gets output, saves
    }

    pub(super) fn get_output<F>(&self, activation_function: F) -> f64 where
        F: Fn(f64) -> f64 {
        let pre_activated_output = self.get_total_in_from_connections();
        return activation_function(pre_activated_output); // return activated output
    }

    fn get_total_in_from_connections(&self) -> f64 {
        let mut total_in: f64 = 0.0;
        for connection in self.connections.get_data() { total_in += connection.get_weighted_connection_value(); }
        total_in
    }

    //for setting the input nodes' values
    pub(super) fn set_output(&mut self, output: Option<f64>) {
        self.output = output;
    }

    pub(super) fn new_node_ref_with_refcell_from_x(x: f64) -> Rc<RefCell<Node>> {
        Rc::new(RefCell::new(Node::new(x)))
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
        if self.x > other.x { Ordering::Greater } else if self.x < other.x { Ordering::Less } else { Ordering::Equal }
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.connections.get_data() == other.connections.get_data() && self.output == other.output && self.x == other.x
    }
}

impl Eq for Node {}