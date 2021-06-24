use std::rc::Rc;
use super::genome::Genome;
use super::node_gene::NodeGene;
use super::connection_gene::ConnectionGene;
use super::calculator::Calculator;
use super::gene::Gene;
use crate::activation_functions::ActivationFunctions;

pub fn get_testing_genome_0() -> Genome {
    //create genome
    let mut genome: Genome = Genome::new();

    //add nodes
    genome.add_node(Rc::new(NodeGene::new(2, 0.9, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(1, 0.1, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(4, 0.9, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(3, 0.9, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(6, 0.5, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(5, 0.9, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(0, 0.1, 0.1)));

    //add connections
    let mut con0 = ConnectionGene::new(0, Rc::clone(&genome.nodes.get(&0).unwrap()), Rc::clone(&genome.nodes.get(&2).unwrap()));
    con0.weight = 0.5;
    let mut con1 = ConnectionGene::new(1, Rc::clone(&genome.nodes.get(&1).unwrap()), Rc::clone(&genome.nodes.get(&3).unwrap()));
    con1.weight = 0.5;
    let mut con2 = ConnectionGene::new(2, Rc::clone(&genome.nodes.get(&0).unwrap()), Rc::clone(&genome.nodes.get(&6).unwrap()));
    con2.weight = 2.0;
    let mut con3 = ConnectionGene::new(3, Rc::clone(&genome.nodes.get(&6).unwrap()), Rc::clone(&genome.nodes.get(&4).unwrap()));
    con3.weight = 1.0;
    let mut con4 = ConnectionGene::new(4, Rc::clone(&genome.nodes.get(&1).unwrap()), Rc::clone(&genome.nodes.get(&6).unwrap()));
    con4.weight = 1.0;
    let mut con5 = ConnectionGene::new(5, Rc::clone(&genome.nodes.get(&6).unwrap()), Rc::clone(&genome.nodes.get(&5).unwrap()));
    con5.weight = 0.75;

    genome.connections.insert(con0.get_innovation_number(),con0);
    genome.connections.insert(con1.get_innovation_number(),con1);
    genome.connections.insert(con2.get_innovation_number(),con2);
    genome.connections.insert(con3.get_innovation_number(),con3);
    genome.connections.insert(con4.get_innovation_number(),con4);
    genome.connections.insert(con5.get_innovation_number(),con5);

    genome
}

pub fn get_testing_genome_1() -> Genome {
    //create genome
    let mut genome: Genome = Genome::new();

    //add nodes
    genome.add_node(Rc::new(NodeGene::new(0, 0.1, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(1, 0.1, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(2, 0.9, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(3, 0.9, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(4, 0.9, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(5, 0.9, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(6, 0.5, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(7, 0.5, 0.1)));
    genome.add_node(Rc::new(NodeGene::new(8, 0.5, 0.1)));


    let edges: [(usize, usize); 7] = [(0, 6), (6,2), (6,4), (1,7), (7,8), (8,3), (8,5)];
    let weights: [f64; 7] = [1.72, 0.5, 1.5, 1.0, 10.0, 0.001, 5.27];

    let mut con_i = 0;

    while con_i < edges.len() {
        let mut con = ConnectionGene::new(con_i, Rc::clone(&genome.nodes.get(&edges[con_i].0).unwrap()), Rc::clone(&genome.nodes.get(&edges[con_i].1).unwrap()));
        con.weight = weights[con_i];
        genome.connections.insert(con.get_innovation_number(), con);

        con_i += 1;
    }


    genome
}

#[test]
fn test_calculator() {
    let mut genome = get_testing_genome_0();

    //create calculator, identity activation function
    let calc = Calculator::new(&genome, |value| -> f64 {
        ActivationFunctions::identity(value)
    });

    let output = calc.run(&vec![1.0, 1.0]);

    println!("Identity output: {:?}", output);

    //tests
    assert_eq!(*output.get(0).unwrap(), 0.5, "testing with identity activation function");
    assert_eq!(*output.get(1).unwrap(), 0.5, "testing with identity activation function");
    assert_eq!(*output.get(2).unwrap(), 3.0, "testing with identity activation function");
    assert_eq!(*output.get(3).unwrap(), 2.25, "testing with identity activation function");


    //create calculator, scuffed sigmoid activation function - as defined by 1.0 / (1.0 + (0.0-input).exp())
    let calc = Calculator::new(&genome, |value| -> f64 {
        ActivationFunctions::scuffed_sigmoid(value)
    });

    let output = calc.run(&vec![1.0, 1.0]);

    println!("Scuffed sigmoid output: {:?}", output);

    //tests
    assert_eq!(*output.get(0).unwrap(), ActivationFunctions::scuffed_sigmoid(0.5), "Output 0: testing with scuffed sigmoid activation function");
    assert_eq!(*output.get(1).unwrap(), ActivationFunctions::scuffed_sigmoid(0.5), "Output 1: testing with scuffed sigmoid activation function");
    assert_eq!(*output.get(2).unwrap(), ActivationFunctions::scuffed_sigmoid(ActivationFunctions::scuffed_sigmoid(2.0 * 1.0 + 1.0 * 1.0) * 1.0), "Output 2: testing with scuffed sigmoid activation function");
    assert_eq!(*output.get(3).unwrap(), ActivationFunctions::scuffed_sigmoid(ActivationFunctions::scuffed_sigmoid(2.0 * 1.0 + 1.0 * 1.0) * 0.75), "Output 3: testing with scuffed sigmoid activation function");
}