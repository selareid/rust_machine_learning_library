use crate::feed_forward::tests::*;
use std::collections::HashMap;
use crate::feed_forward::genome::Genome;
use super::genome_neat::{GenomeNeatMethods, GenomeMutator};
use super::Neat;
use crate::feed_forward::gene::Gene;
use crate::feed_forward::connection_gene::ConnectionGene;
use crate::activation_functions::ActivationFunctions;

fn get_neat_for_tests() -> Neat {
    Neat::new(5, 2, ActivationFunctions::identity, 0.0,
              0, 0, 0, 0, 0, 0.0)
}
fn get_basic_genome_for_test_using_neat(neat: &mut Neat) -> Genome {
    //create genome
    let mut genome: Genome = Genome::new();

    //add nodes
    for inv_num in 0..neat.node_bank.len() {
        if !neat.node_bank.contains_key(&inv_num) {
            break;
        }

        let node_opt = neat.get_node_by_inv_num(inv_num);

        match node_opt {
            None => break,
            Some(node) => {
                if node.get_x() != 0.1 && node.get_x() != 0.9 {
                    break;
                }

                genome.add_node(node);
            }
        }
    }
    let weights: [f64; 7] = [1.72, 0.5, 1.5, 1.0, 10.0, 0.001, 5.27];

    //generate connections
    for i in 1..neat.num_of_input_nodes {
        let out_node_opt = neat.get_node_by_inv_num(neat.node_bank.len()-i);

        match out_node_opt {
            None => break,
            Some(out_node) => {
                if out_node.get_x() != 0.9 {
                    break
                }

                let in_node = neat.get_node_by_inv_num(i).unwrap();

                let mut con = ConnectionGene::new(neat.get_connection_number_from_nodes(in_node.get_innovation_number(), out_node.get_innovation_number()), in_node, out_node);

                con.weight = weights[con.get_innovation_number() % weights.len()];

                genome.add_connection(con);
            }
        }
    }
    println!("On Creation,\n connections: {:?}\n nodes: {:?}", genome.connections.keys(), genome.nodes.keys());

    genome
}
//TODO note - could implement (count key different) for testing - maybe use a macro and accept closure with some stuff to do on-loop

#[test]
fn test_distance_function() {
    todo!()
}

#[test]
fn test_breeding() {
    let genome0 = get_testing_genome_0();
    let genome1 = get_testing_genome_1();
    let breeded = GenomeNeatMethods::breed(&genome1, &genome0);

    println!("TESTED BREEDING");

    for (_key, connection) in breeded.connections {
        println!("Connection {:?}", connection);
    }

    for (_key, node_ref) in breeded.nodes {
        let node = node_ref.as_ref();
        println!("Node {:?}", node);
    }

    //TODO this doesn't really check breeding, though idk how i'd check that so meh...??
}

#[test]
fn test_mutate_add_node() {
    let mut neat = get_neat_for_tests();

    let mut genome0 = get_basic_genome_for_test_using_neat(&mut neat);

    let original_con_keys: Vec<usize> = genome0.connections.keys().map(|v: &usize| *v).collect();
    let original_node_keys: Vec<usize> = genome0.nodes.keys().map(|v: &usize| *v).collect();

    println!("pre-mutate node connection map keys {:?}", neat.nodes_to_connection_map.keys());

    //do mutate
    assert!(GenomeMutator::mutate_add_node(&mut neat, &mut genome0));

    println!("post-mutate node connection map keys {:?}", neat.nodes_to_connection_map.keys());

    let mut con_difference: usize = 0;

    for key in genome0.connections.keys() {
        if !original_con_keys.contains(key) {
            con_difference += 1;
        }
    }

    assert_eq!(con_difference, 2);

    let mut node_difference: usize = 0;

    for key in genome0.nodes.keys() {
        if !original_node_keys.contains(key) {
            node_difference += 1;
        }
    }

    assert_eq!(node_difference, 1);
}

#[test]
fn test_mutate_add_connection() {
    let mut neat = get_neat_for_tests();

    let mut genome0 = get_basic_genome_for_test_using_neat(&mut neat);

    let original_con_keys: Vec<usize> = genome0.connections.keys().map(|v: &usize| *v).collect();
    let original_node_keys: Vec<usize> = genome0.nodes.keys().map(|v: &usize| *v).collect();

    println!("pre-mutate node connection map keys {:?}", neat.nodes_to_connection_map.keys());

    //do mutate
    assert!(GenomeMutator::mutate_add_connection(&mut neat, &mut genome0));

    println!("post-mutate node connection map keys {:?}", neat.nodes_to_connection_map.keys());

    let mut con_difference: usize = 0;

    for key in genome0.connections.keys() {
        if !original_con_keys.contains(key) {
            con_difference += 1;
        }

        assert!(genome0.connections.get(key).unwrap().from.get_x() < genome0.connections.get(key).unwrap().to.get_x());
    }

    assert_eq!(con_difference, 1);

    let mut node_difference: usize = 0;

    for key in genome0.nodes.keys() {
        if !original_node_keys.contains(key) {
            node_difference += 1;
        }
    }

    assert_eq!(node_difference, 0);
}

#[test]
fn test_mutate_random_weight() {
    //TODO - check that the weight is within the max bound

    let mut neat = get_neat_for_tests();

    let mut genome0 = get_basic_genome_for_test_using_neat(&mut neat);

    let original_con_keys: Vec<usize> = genome0.connections.keys().map(|v: &usize| *v).collect();
    let original_con_weights: HashMap<usize, f64> = genome0.connections.iter().map(|(key, con): (&usize, &ConnectionGene)| (*key, con.weight)).collect();
    let original_node_keys: Vec<usize> = genome0.nodes.keys().map(|v: &usize| *v).collect();

    //do mutate
    assert!(GenomeMutator::mutate_random_weight(&mut neat, &mut genome0));

    //check that the connections and nodes are same (by inv_num) and also checks weight diff
    let mut con_difference: usize = 0;
    let mut con_weight_difference: usize = 0;

    for (key, con) in genome0.connections.iter() {
        if !original_con_keys.contains(key) {
            con_difference += 1;
        }
        else if &con.weight != original_con_weights.get(key).unwrap() {
            con_weight_difference += 1;
        }

        assert!(genome0.connections.get(key).unwrap().from.get_x() < genome0.connections.get(key).unwrap().to.get_x());
    }

    assert_eq!(con_difference, 0);
    assert_eq!(con_weight_difference, 1);

    let mut node_difference: usize = 0;

    for key in genome0.nodes.keys() {
        if !original_node_keys.contains(key) {
            node_difference += 1;
        }
    }

    assert_eq!(node_difference, 0);
}

#[test]
fn test_mutate_weight_shift() {
    //TODO - check that the difference in weight is within the max bound

    let mut neat = get_neat_for_tests();

    let mut genome0 = get_basic_genome_for_test_using_neat(&mut neat);

    let original_con_keys: Vec<usize> = genome0.connections.keys().map(|v: &usize| *v).collect();
    let original_con_weights: HashMap<usize, f64> = genome0.connections.iter().map(|(key, con): (&usize, &ConnectionGene)| (*key, con.weight)).collect();
    let original_node_keys: Vec<usize> = genome0.nodes.keys().map(|v: &usize| *v).collect();

    //do mutate
    assert!(GenomeMutator::mutate_weight_shift(&mut neat, &mut genome0));

    //check that the connections and nodes are same (by inv_num) and also checks weight diff
    let mut con_difference: usize = 0;
    let mut con_weight_difference: usize = 0;

    for (key, con) in genome0.connections.iter() {
        if !original_con_keys.contains(key) {
            con_difference += 1;
        }
        else if &con.weight != original_con_weights.get(key).unwrap() {
            con_weight_difference += 1;
        }

        assert!(genome0.connections.get(key).unwrap().from.get_x() < genome0.connections.get(key).unwrap().to.get_x());
    }

    assert_eq!(con_difference, 0);
    assert_eq!(con_weight_difference, 1);

    let mut node_difference: usize = 0;

    for key in genome0.nodes.keys() {
        if !original_node_keys.contains(key) {
            node_difference += 1;
        }
    }

    assert_eq!(node_difference, 0);
}

#[test]
fn test_mutate_toggle_connection() {
    let mut neat = get_neat_for_tests();

    let mut genome0 = get_basic_genome_for_test_using_neat(&mut neat);

    let original_con_keys: Vec<usize> = genome0.connections.keys().map(|v: &usize| *v).collect();
    let original_con_enabled: HashMap<usize, bool> = genome0.connections.iter().map(|(key, con): (&usize, &ConnectionGene)| (*key, con.enabled)).collect();
    let original_node_keys: Vec<usize> = genome0.nodes.keys().map(|v: &usize| *v).collect();

    //do mutate
    assert!(GenomeMutator::mutate_toggle_connection(&mut neat, &mut genome0));

    //check that the connections and nodes are same (by inv_num) and also checks weight diff
    let mut con_difference: usize = 0;
    let mut con_enabled_difference: usize = 0;

    for (key, con) in genome0.connections.iter() {
        if !original_con_keys.contains(key) {
            con_difference += 1;
        }
        else if &con.enabled != original_con_enabled.get(key).unwrap() {
            con_enabled_difference += 1;
        }

        assert!(genome0.connections.get(key).unwrap().from.get_x() < genome0.connections.get(key).unwrap().to.get_x());
    }

    assert_eq!(con_difference, 0);
    assert_eq!(con_enabled_difference, 1);

    let mut node_difference: usize = 0;

    for key in genome0.nodes.keys() {
        if !original_node_keys.contains(key) {
            node_difference += 1;
        }
    }

    assert_eq!(node_difference, 0);
}