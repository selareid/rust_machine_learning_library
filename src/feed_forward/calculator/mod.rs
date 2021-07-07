use std::cell::{Ref, RefCell};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::rc::Rc;

use crate::feed_forward::calculator::connection::Connection;
use crate::feed_forward::calculator::node::Node;
use crate::feed_forward::connection_gene::ConnectionGene;
use crate::feed_forward::gene::Gene;
use crate::feed_forward::genome::Genome;

pub(crate) mod connection;
pub(crate) mod node;

pub(crate) struct Calculator<F> where
    F: Fn(f64) -> f64 {
    input_nodes: Vec<Rc<RefCell<Node>>>,
    output_nodes: Vec<Rc<RefCell<Node>>>,
    hidden_nodes: Vec<Rc<RefCell<Node>>>, //order of these can change

    input_innovation_num_to_position: HashMap<usize, usize>,
    output_inv_to_position: HashMap<usize, usize>,

    activation_function: F,
}

impl<F> Calculator<F> where
    F: Fn(f64) -> f64 {
    fn new_with_defaults(activation_function: F) -> Calculator<F> {
        Calculator {
            input_nodes: Vec::new(),
            hidden_nodes: Vec::new(),
            output_nodes: Vec::new(),
            input_innovation_num_to_position: HashMap::new(),
            output_inv_to_position: HashMap::new(),
            activation_function,
        }
    }

    pub fn new_from_ref(genome_ref: Rc<RefCell<Genome>>, activation_function: F) -> Self {
        let genome: Ref<Genome> = genome_ref.borrow();
        Self::new(&*genome, activation_function)
    }

    pub fn new(genome: &Genome, activation_function: F) -> Self {
        let mut new_calculator = Self::new_with_defaults(activation_function);
        new_calculator.add_genome_data_to_calculator(genome);
        new_calculator
    }

    fn add_genome_data_to_calculator(&mut self, genome: &Genome) {
        let mut node_innovation_num_to_ref: HashMap<usize, Rc<RefCell<Node>>> = HashMap::new(); //so that the same nodes are the same (not copies/clones)

        //start of node stuffs
        for (_inv_num, genome_node) in genome.nodes.iter() {
            let node: Rc<RefCell<Node>> = Node::new_node_ref_with_refcell_from_x(genome_node.get_x());

            node_innovation_num_to_ref.insert(genome_node.get_innovation_number(), Rc::clone(&node));
            self.add_node_to_calculator(node, genome_node.get_innovation_number());
        }

        //start of connections stuff
        for (_key, genome_connection) in &genome.connections {
            if !genome_connection.enabled { continue; }

            Self::add_connection_to_nodes_from_gene(&mut node_innovation_num_to_ref, genome_connection);
        }

        self.hidden_nodes.sort_unstable();
    }

    fn add_connection_to_nodes_from_gene(mut node_innovation_num_to_ref_map: &mut HashMap<usize, Rc<RefCell<Node>>>, connection_gene: &ConnectionGene) {
        let (from_node, to_node) = get_node_refs_from_connection_gene(&mut node_innovation_num_to_ref_map, &connection_gene);

        let new_connection: Connection = Connection::new(connection_gene.weight, from_node);

        &mut to_node.borrow_mut().connections.push(Rc::new(new_connection));
    }

    fn add_node_to_calculator(&mut self, node_ref: Rc<RefCell<Node>>, innovation_number: usize) {
        let node_x: f64 = node_ref.borrow().x;

        let is_hidden_node: bool = node_x > 0.1 && node_x < 0.9;
        if !is_hidden_node {
            let node_vector_length = self.get_node_vector_from_x(node_x).len();
            self.get_node_to_position_map_from_x_mut(node_x).insert(innovation_number, node_vector_length);
        }

        self.get_node_vector_from_x_mut(node_x).push(node_ref);
    }

    fn get_node_vector_from_x_mut(&mut self, node_x: f64) -> &mut Vec<Rc<RefCell<Node>>> {
        if node_x <= 0.1 { //input node
            &mut self.input_nodes
        } else if node_x >= 0.9 { //output node
            &mut self.output_nodes
        } else { //hidden node
            &mut self.hidden_nodes
        }
    }

    fn get_node_vector_from_x(&mut self, node_x: f64) -> &Vec<Rc<RefCell<Node>>> {
        self.get_node_vector_from_x_mut(node_x)
    }

    fn get_node_to_position_map_from_x_mut(&mut self, node_x: f64) -> &mut HashMap<usize, usize> {
        if node_x <= 0.1 { //input node
            &mut self.input_innovation_num_to_position
        } else if node_x >= 0.9 { //output node
            &mut self.output_inv_to_position
        } else {
            panic!("whoa, that's illegal");
        }
    }

    //assumes that input vector maps to innovation number (0th in inputs = node with inv num 0)
    pub fn run(&self, inputs: &Vec<f64>) -> Vec<f64> {
        if inputs.len() < self.input_nodes.len() { panic!("BAD INPUT TO CALCULATOR"); }

        self.process_input_nodes(inputs);
        self.process_hidden_nodes();
        self.get_outputs_from_nodes()
    }

    fn get_outputs_from_nodes(&self) -> Vec<f64> {
        let mut outputs: Vec<f64> = Vec::new();
        self.process_output_nodes(&mut outputs);
        outputs
    }

    fn process_input_nodes(&self, inputs: &Vec<f64>) {
        //map input arguments to input_node's output
        for node_innovation_number in 0..self.input_nodes.len() {
            let node_position = self.input_innovation_num_to_position.get(&node_innovation_number).unwrap();
            let node: &mut Node = &mut self.input_nodes[*node_position].borrow_mut();

            node.set_output(Some(inputs[node_innovation_number]));
        }
    }

    fn process_hidden_nodes(&self) {
        for node_i in 0..self.hidden_nodes.len() {
            let node: &mut Node = &mut self.hidden_nodes[node_i].borrow_mut();

            node.run_node(&self.activation_function);
        }
    }

    fn process_output_nodes(&self, outputs: &mut Vec<f64>) {
        for i in (self.input_nodes.len()..).take(self.output_nodes.len()) {
            let node_position = self.output_inv_to_position.get(&i).unwrap();
            let node_rc: &Rc<RefCell<Node>> = &self.output_nodes[*node_position];
            node_rc.borrow_mut().run_node(&self.activation_function);

            outputs.push(node_rc.borrow().output.unwrap());
        }
    }
}

fn get_node_refs_from_connection_gene(node_innovation_num_to_ref: &mut HashMap<usize, Rc<RefCell<Node>>>, connection_gene: &&ConnectionGene) -> (Rc<RefCell<Node>>, Rc<RefCell<Node>>) {
    let from_node: Rc<RefCell<Node>> = match node_innovation_num_to_ref.get(&connection_gene.from.get_innovation_number()) {
        None => panic!(),
        Some(node) => Rc::clone(node)
    };

    let to_node: Rc<RefCell<Node>> = match node_innovation_num_to_ref.get(&connection_gene.to.get_innovation_number()) {
        None => panic!(),
        Some(node) => Rc::clone(node)
    };

    (from_node, to_node)
}