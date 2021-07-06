use super::node::Node;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Connection {
    pub weight: f64,
    pub node_ref: Rc<RefCell<Node>>,
}

impl Connection {
    pub(super) fn new(weight: f64, node: Rc<RefCell<Node>>) -> Self{
        Connection {weight, node_ref: node }
    }

    pub(super) fn get_weighted_connection_value(&self) -> f64 {
        let node = self.node_ref.borrow();
        match node.output {
            None => panic!("ERROR, no output in input node - illegal for this basic feed-forward network"),
            Some(output) => {
                output * self.weight
            }
        }
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight && Rc::ptr_eq(&self.node_ref, &other.node_ref)
    }
}

impl Eq for Connection {}