use rand::prelude::*;
use crate::feed_forward::gene::Gene;
use crate::feed_forward::genome::Genome;
use crate::feed_forward::connection_gene::ConnectionGene;
use super::Neat;
use std::rc::Rc;

pub(super) struct GenomeNeatMethods {}

//TODO -- write tests for these
impl GenomeNeatMethods {
    //uses the function given in the original NEAT paper
    pub(crate) fn distance(genome0: &Genome, genome1: &Genome, c_constants: (f64, f64, f64)) -> f64 {
        //set g1, g2 where g1 has the highest innovation number
        let g0: &Genome;
        let g1: &Genome;

        if genome0.connections.len() > genome1.connections.len() {
            g0 = genome0;
            g1 = genome1;
        } else {
            g1 = genome0;
            g0 = genome1;
        }

        let mut index_g0: usize = 0;
        let mut index_g1: usize = 0;

        let mut disjoint: usize = 0;
        let excess: usize;
        let mut weight_diff: f64 = 0.0;
        let mut similar: usize = 0;

        //count
        while index_g0 < g0.connections.len() && index_g1 < g1.connections.len() {
            if let Some(connection_g0) = g0.connections.get(&index_g0) {
                if let Some(connection_g1) = g1.connections.get(&index_g1) {
                    if connection_g0.get_innovation_number() == connection_g1.get_innovation_number() { //similar gene
                        //update counts
                        similar += 1;
                        weight_diff += (connection_g0.weight - connection_g1.weight).abs();
                        //increase indices
                        index_g0 += 1;
                        index_g1 += 1;
                    } else if connection_g0.get_innovation_number() > connection_g1.get_innovation_number() { //disjoint gene of b
                        disjoint += 1; //update count
                        index_g1 += 1; //increase lower index
                    } else { //disjoint of a
                        disjoint += 1; //update count
                        index_g0 += 1; //increase lower index
                    }
                }
            }
        }

        weight_diff /= (1.max(similar)) as f64; // calculate the average
        excess = g0.connections.len() - index_g0; // count excess genes

        //YEET INTO THE MAGIC EQUATION
        let pre_n: usize = g0.connections.len().max(g1.connections.len());
        let n: f64;

        if pre_n < 20 {
            n = 1.0;
        } else {
            n = pre_n as f64;
        }

        //return the distance
        (c_constants.0 * excess as f64 / n
            + c_constants.1 * disjoint as f64 / n
            + c_constants.2 * weight_diff) as f64
    }

    //where genome0 is fitter than genome1
    pub(crate) fn breed(genome0: &Genome, genome1: &Genome) -> Genome {
        let mut new_genome: Genome = Genome::new();

        let mut index_g0: usize = 0;
        let mut index_g1: usize = 0;

        let mut rng = rand::thread_rng(); //caching the generator for performance reasons

        while index_g0 < genome0.connections.len() && index_g1 < genome1.connections.len() {
            if let Some(connection_g0) = genome0.connections.get(&index_g0) {
                if let Some(connection_g1) = genome1.connections.get(&index_g1) {
                    if connection_g0.get_innovation_number() == connection_g1.get_innovation_number() { //similar gene
                        //increase indices
                        index_g0 += 1;
                        index_g1 += 1;

                        //randomly copy connection from one of the given genomes
                        if rng.gen::<bool>() {
                            new_genome.connections.insert(connection_g0.get_innovation_number(), ConnectionGene::clone(connection_g0));
                        } else {
                            new_genome.connections.insert(connection_g1.get_innovation_number(), ConnectionGene::clone(connection_g1));
                        }
                    } else if connection_g0.get_innovation_number() > connection_g1.get_innovation_number() { //disjoint gene of b
                        index_g1 += 1; //increase lower index
                    } else { //disjoint of a
                        index_g0 += 1; //increase lower index
                        new_genome.connections.insert(connection_g0.get_innovation_number(), ConnectionGene::clone(connection_g0));
                    }
                }
            }
        }

        //fill in remaining connections
        while index_g0 < genome0.connections.len() {
            if let Some(connection_g0) = genome0.connections.get(&index_g0) {
                new_genome.add_connection(ConnectionGene::clone(connection_g0));
                index_g0 += 1;
            }
        }

        //fill out the nodes in the new genome
        let mut index_in_new: usize = 0;
        while index_in_new < new_genome.connections.len() {
            if let Some(connection) = new_genome.connections.get(&index_in_new) {
                let (from_node, to_node) = (Rc::clone(&connection.from), Rc::clone(&connection.to));

                //handle from node
                new_genome.add_node(from_node);

                //handle to node
                new_genome.add_node(to_node);
            } else {
                panic!("This shouldn't happen I think");
            }

            index_in_new += 1;
        }

        new_genome
    }
}

pub(super) struct GenomeMutator {}

impl GenomeMutator {
    pub(crate) fn mutate_random(neat: &mut Neat, genome: &mut Genome) {
        if (0..neat.mutate_chance_add_node).choose(&mut neat.cached_rng) == Some(0) { GenomeMutator::mutate_add_node(neat, genome); }
        if (0..neat.mutate_chance_add_connection).choose(&mut neat.cached_rng) == Some(0) { GenomeMutator::mutate_add_connection(neat, genome); }
        if (0..neat.mutate_chance_random_weight).choose(&mut neat.cached_rng) == Some(0) { GenomeMutator::mutate_random_weight(neat, genome); }
        if (0..neat.mutate_chance_weight_shift).choose(&mut neat.cached_rng) == Some(0) { GenomeMutator::mutate_weight_shift(neat, genome); }
        if (0..neat.mutate_chance_toggle_connection).choose(&mut neat.cached_rng) == Some(0) { GenomeMutator::mutate_toggle_connection(neat, genome); }

    }

    pub(super) fn mutate_add_node(neat: &mut Neat, genome: &mut Genome) -> bool {
        let mut key: usize = 0;

        if let Some(chosen_key) = genome.connections.keys().choose(&mut neat.cached_rng) {
            key = chosen_key.clone();
        }
            if let Some(con) = genome.connections.get_mut(&key) {
                {
                    if !con.enabled {
                        return false;
                    }
                    con.enabled = false;
                }

                let (new_con0, new_con1, new_node) = neat.get_replacement_for_connection(con);

                genome.add_connection(new_con0);
                genome.add_connection(new_con1);
                genome.add_node(Rc::from(new_node));

                return true
            }

        false
    }

    pub(super) fn mutate_add_connection(neat: &mut Neat, genome: &mut Genome) -> bool {
        let mut i = 0;

        while {
            i += 1;
            if i > neat.get_max_mutation_attempts() {
                return false;
            }

            //gets 2 random keys
            if let (Some(a_node1), Some(a_node2)) = (genome.nodes.keys().choose(&mut neat.cached_rng), genome.nodes.keys().choose(&mut neat.cached_rng)) {
                if a_node1 == a_node2 {
                    true
                } else {
                    //set node0 to node with higher key, and node1 to other
                    if let (Some(node0), Some(node1))
                      = (neat.get_node_by_inv_num(*if a_node1 < a_node2 { a_node1 } else { a_node2 }), neat.get_node_by_inv_num(*if a_node1 > a_node2 { a_node1 } else { a_node2 })) {
                        if node0.get_x() < node1.get_x() {
                            let con_inv_num = neat.get_connection_number_from_nodes(node0.get_innovation_number(), node1.get_innovation_number());
                            if genome.connections.contains_key(&con_inv_num) {
                                true
                            } else {
                                let mut connection = neat.new_connection(con_inv_num, node0.get_innovation_number(), node1.get_innovation_number());
                                connection.weight = neat.cached_rng.gen_range(-(1.0 * neat.get_random_weight_max())..(1.0 * neat.get_random_weight_max()));

                                genome.add_connection(connection);
                                return true;
                            }
                        } else {
                            true
                        }
                    } else {
                        return false;
                    }
                }
            }
            else {
                return false;
            }
        } {}

        false
    }

    //randomly change a connections weight
    pub(super) fn mutate_random_weight(neat: &mut Neat, genome: &mut Genome) -> bool {
        let mut i = 0;

        //scuffed do/while (the stuff is in the while's expression)
        while {
            i += 1;
            if i > neat.get_max_mutation_attempts() {
                return false;
            }

            //gets random connection key
            if let Some(con_key) = genome.connections.keys().choose(&mut neat.cached_rng) {
                let con_key = &con_key.clone(); //because we borrow .connections as immutable then mutable so we need to drop the immutable reference
                if let Some(connection) = genome.connections.get_mut(con_key) {
                    if connection.enabled {
                        connection.weight = neat.cached_rng.gen_range(-(1.0 * neat.get_random_weight_max())..(1.0 * neat.get_random_weight_max()));
                        return true;
                    } else {
                        true
                    }
                } else {
                    return false;
                }
            } else {
                return false;
            }
        } {}

        false
    }

    pub(super) fn mutate_weight_shift(neat: &mut Neat, genome: &mut Genome) -> bool {
        let mut i = 0;

        //scuffed do/while (the stuff is in the while's expression)
        while {
            i += 1;
            if i > neat.get_max_mutation_attempts() {
                panic!("uhhhh fix your code bro");
            }

            //gets random connection key
            if let Some(con_key) = genome.connections.keys().choose(&mut neat.cached_rng) {
                let con_key = &con_key.clone(); //because we borrow .connections as immutable then mutable so we need to drop the immutable reference
                if let Some(connection) = genome.connections.get_mut(con_key) {
                    if connection.enabled {
                        //randomly shift weight
                        connection.weight += neat.cached_rng.gen_range(-(1.0 * neat.get_random_weight_shift_max())..(1.0 * neat.get_random_weight_shift_max()));
                        return true;
                    }
                    else {
                        true
                    }
                }
                else {
                    return false;
                }
            }
            else {
                return false;
            }
        } {}

        false
    }

    //change connection enabled/disabled
    pub(super) fn mutate_toggle_connection(neat: &mut Neat, genome: &mut Genome) -> bool {
        //gets random connection key
        if let Some(con_key) = genome.connections.keys().choose(&mut neat.cached_rng) {
            let con_key = &con_key.clone(); //because we borrow .connections as immutable then mutable so we need to drop the immutable reference
            if let Some(connection) = genome.connections.get_mut(con_key) {
                connection.enabled = !connection.enabled;
                return true;
            }
        }

        return false;
    }
}