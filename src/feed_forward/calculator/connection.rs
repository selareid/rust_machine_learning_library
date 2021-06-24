use super::node::Node;
use std::rc::Rc;
use std::cell::RefCell;

pub struct Connection {
    pub weight: f64,
    pub node: Rc<RefCell<Node>>,
}

impl Connection {
    pub fn new(weight: f64, node: Rc<RefCell<Node>>) -> Self{
        Connection {weight, node}
    }
}

impl PartialEq for Connection {
    fn eq(&self, other: &Self) -> bool {
        self.weight == other.weight && Rc::ptr_eq(&self.node, &other.node)
    }
}

impl Eq for Connection {}