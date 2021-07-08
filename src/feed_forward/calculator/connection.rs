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


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_connection_has_weight_and_node_given() {
        let weight = 55_f64;
        let node_ref = Rc::new(RefCell::new(Node::new(0.5)));

        let new_connection = Connection::new(weight, Rc::clone(&node_ref));

        assert_eq!(new_connection.weight, weight);
        assert!(Rc::ptr_eq(&new_connection.node_ref, &node_ref));
    }

    #[test]
    #[should_panic]
    fn get_weighted_connection_value_panics_with_new_node() {
        let weight = 55_f64;
        let node_ref = Rc::new(RefCell::new(Node::new(0.5)));

        let new_connection = Connection::new(weight, Rc::clone(&node_ref));

        new_connection.get_weighted_connection_value();
    }

    #[test]
    fn get_weighted_connection_value_returns_as_expected_with_doctored_node_output() {
        let weight = 55_f64;
        let node_ref = Rc::new(RefCell::new(Node::new(0.5)));

        let node_output = 0.22_f64;
        node_ref.borrow_mut().set_output(Some(node_output));

        let new_connection = Connection::new(weight, Rc::clone(&node_ref));

        let return_value = new_connection.get_weighted_connection_value();
        assert_eq!(return_value, weight * node_output);
    }
}