use crate::feed_forward::calculator::node::Node;
use crate::feed_forward::genome::Genome;
use std::collections::HashMap;
use std::rc::Rc;
use crate::feed_forward::calculator::connection::Connection;
use std::cell::{RefCell, Ref};
use crate::feed_forward::gene::Gene;

pub(crate) mod connection;
pub(crate) mod node;

pub(crate) struct Calculator<F> where
    F: Fn(f64) -> f64 {
    input_nodes: Vec<Rc<RefCell<Node>>>,
    output_nodes: Vec<Rc<RefCell<Node>>>,
    hidden_nodes: Vec<Rc<RefCell<Node>>>, //order of this one can change

    input_inv_to_position: HashMap<usize, usize>,
    output_inv_to_position: HashMap<usize, usize>,

    activation_function: F,
}

impl<F> Calculator<F> where
    F: Fn(f64) -> f64 {
    pub fn new_from_ref(genome_ref: Rc<RefCell<Genome>>, activation_function: F) -> Self {
        let genome: Ref<Genome> = genome_ref.borrow();
        Calculator::new(&*genome, activation_function)
    }

    pub fn new(genome: &Genome, activation_function: F) -> Self {
        let mut calculator = Calculator {
            input_nodes: Vec::new(), hidden_nodes: Vec::new(), output_nodes: Vec::new(),
            input_inv_to_position: HashMap::new(), output_inv_to_position: HashMap::new(),
            activation_function};

        let mut node_hash_map: HashMap<usize, Rc<RefCell<Node>>> = HashMap::new();

        //start of node stuffs
        for (_inv_num, genome_node) in genome.nodes.iter() {

            let node: Rc<RefCell<Node>> = Rc::new(RefCell::new(Node::new(genome_node.get_x())));

            node_hash_map.insert(genome_node.get_innovation_number(), Rc::clone(&node));

            if genome_node.get_x() <= 0.1 {
                calculator.input_inv_to_position.insert(genome_node.get_innovation_number(), calculator.input_nodes.len());
                calculator.input_nodes.push(node);
            }
            else if genome_node.get_x() >= 0.9 {
                calculator.output_inv_to_position.insert(genome_node.get_innovation_number(), calculator.output_nodes.len());
                calculator.output_nodes.push(node);
            }
            else {
                calculator.hidden_nodes.push(node);
            }
        }

        //start of connections stuff
        for (_key, genome_connection) in &genome.connections {
            if !genome_connection.enabled {
                continue;
            }

            let from_node: &Rc<RefCell<Node>> = match node_hash_map.get(&genome_connection.from.get_innovation_number()) {
                None => panic!("Failed to get node with inv number {} from node_hash_map", &genome_connection.from.get_innovation_number()),
                Some(node) => node
            };

            let to_node: &Rc<RefCell<Node>> = match node_hash_map.get(&genome_connection.to.get_innovation_number()) {
                None => panic!("Failed to get node with inv number {} from node_hash_map {:?}", &genome_connection.to.get_innovation_number(), node_hash_map.keys()),
                Some(node) => node
            };

            let new_connection: Connection = Connection::new(genome_connection.weight, Rc::clone(from_node));

            &mut to_node.borrow_mut().connections.push(Rc::new(new_connection));
        }

        calculator.hidden_nodes.sort_unstable();

        calculator
    }

    //assumes that input vector maps to innovation number (0th in inputs = node with inv num 0)
    pub fn run(&self, inputs: &Vec<f64>) -> Vec<f64> {
        if inputs.len() < self.input_nodes.len() {
            panic!("BAD INPUT TO CALCULATOR");
        }

        //process input nodes
        //map input arguments to input_node's output
        for node_inv in 0..self.input_nodes.len() {
            let node_i = self.input_inv_to_position.get(&node_inv).unwrap();
            let node: &mut Node = &mut self.input_nodes[*node_i].borrow_mut();
            node.set_output(Some(inputs[node_inv]));
            // println!("Input to node: {} output set to: {:?}", node_i, node.output); //for testing purposes I suppose
        }

        //process hidden nodes
        for node_i in 0..self.hidden_nodes.len() {
            let node: &mut Node = &mut self.hidden_nodes[node_i].borrow_mut();
            node.run_node(&self.activation_function);
            // println!("hidden node: {} output set to: {:?}", node_i, node.output); //for testing purposes I suppose
        }

        //process output nodes
        let mut outputs: Vec<f64> = Vec::new();

        for i in (self.input_nodes.len()..).take(self.output_nodes.len()) {
            let node_position = self.output_inv_to_position.get(&i).unwrap();
            let node_rc: &Rc<RefCell<Node>> = &self.output_nodes[*node_position];
            let node_output = node_rc.borrow().get_output(&self.activation_function);
            outputs.push(node_output);
        }

        return outputs;
    }
}