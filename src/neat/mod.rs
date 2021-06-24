use crate::neat::species::Species;
use std::cell::{RefCell};
use crate::neat::client::Client;
use std::rc::Rc;
use crate::feed_forward::node_gene::NodeGene;
use crate::feed_forward::connection_gene::ConnectionGene;
use std::collections::HashMap;
use crate::feed_forward::gene::Gene;
use crate::feed_forward::genome::Genome;
use crate::neat::genome_neat::GenomeMutator;
use rand::rngs::ThreadRng;
use std::cmp::Ordering::Equal;
use crate::random_hash_set::RandomHashSet;
use rand::prelude::IteratorRandom;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod genome_neat_tests;

mod client;
mod species;
mod genome_neat;

/* - Neat -
everything
 */

pub struct Neat {
    species: HashMap<String, Rc<RefCell<Species>>>, //does not include default_species
    clients: HashMap<String, Rc<RefCell<Client>>>,
    default_species: Rc<RefCell<Species>>, //when a client doesn't have a species

    activation_function: fn(f64) -> f64,
    species_distance_threshold: f64,
    proportion_to_kill: f64, //proportion to kill each run

    //mutate chances - 1 in x chance of mutating
    mutate_chance_add_node: u64,
    mutate_chance_add_connection: u64,
    mutate_chance_random_weight: u64,
    mutate_chance_weight_shift: u64,
    mutate_chance_toggle_connection: u64,

    node_bank: HashMap<usize, Rc<NodeGene>>, //for making sure nodes with same inv_num always refers to same node
    nodes_to_connection_map: HashMap<(usize, usize), usize>, //(node0_inv_num, node1_inv_num) -> connection_inv_num
    connection_to_replacement_node_map: HashMap<usize, usize>, //conection_inv_num -> node_inv_number

    pub(crate) num_of_input_nodes: usize,
    pub(crate) num_of_output_nodes: usize,

    cached_rng: ThreadRng,
}

impl Neat {
    pub fn new(input_size: usize, output_size: usize,
               activation_function: fn(f64) -> f64,
               species_distance_threshold: f64,
               mutate_chance_add_node: u64,
               mutate_chance_add_connection: u64,
               mutate_chance_random_weight: u64,
               mutate_chance_weight_shift: u64,
               mutate_chance_toggle_connection: u64,
               proportion_to_kill: f64,) -> Self {
        let input_size = input_size+1; //add bias node

        let mut neat = Neat {
            species: Default::default(),
            clients: Default::default(),
            default_species: Rc::new(RefCell::new(Species::new())), //generate this or something

            activation_function,
            species_distance_threshold,
            proportion_to_kill,
            mutate_chance_add_node,
            mutate_chance_add_connection,
            mutate_chance_random_weight,
            mutate_chance_weight_shift,
            mutate_chance_toggle_connection,

            node_bank: Default::default(),
            nodes_to_connection_map: Default::default(),
            connection_to_replacement_node_map: Default::default(),

            num_of_input_nodes: input_size,
            num_of_output_nodes: output_size,
            cached_rng: rand::thread_rng(),
        };

        //we add a node (node0) as the bias
        for i in 0..input_size {
            neat.get_new_node_from_xy(0.1, (i / input_size) as f64);
        }
        for i in 0..output_size {
            neat.get_new_node_from_xy(0.9, (i / output_size) as f64);
        }

        neat
    }

    //creates new client, adds to default species
    //returns client's id/name
    //client is ready-to-run on creation (has calculator)
    pub fn new_client(&mut self) -> String {
        //get a basic client
        let mut client = Client::new(Rc::new(RefCell::new(self.get_default_genome())),
                                 self.get_default_species(),
                                 );
        //update calculator
        client.generate_calculator(self.activation_function);

        let name = String::clone(client.get_name());

        let client_ref = Rc::new(RefCell::new(client));

        self.get_default_species().borrow_mut().force_put(Rc::clone(&client_ref), self.get_default_species());

        self.clients.insert(String::clone(&name), client_ref);


        name
    }

    //run the client's calculator
    pub fn use_client(&self, client_name: String, inputs: &Vec<f64>) -> Vec<f64> {
        assert_eq!(inputs.len(), self.num_of_input_nodes-1);
        match self.clients.get(&client_name) {
            None => panic!("Illegal moment, client with name {} does not exist", client_name),
            Some(client_ref) => {
                let mut inputs_with_bias: Vec<f64> = vec![1.0];
                inputs_with_bias.extend(inputs);
                client_ref.borrow().use_calculator(&inputs_with_bias)
            },
        }
    }

    pub fn score_client(&self, client_name: String, score: f64) {
        match self.clients.get(&client_name) {
            None => panic!("Whoa, client with name {} doesn't exist", client_name),
            Some(client_ref) => {
                client_ref.borrow_mut().set_score(score);
            },
        }
    }

    pub fn update_clients(&mut self) {
        /*
        evaluate species
        kill low species
        remove empty species
        */
        {
            let mut score_list: Vec<(String, f64)> = Vec::new();

            //evaluate species
            for (_key, species_ref) in &self.species {
                let mut species = species_ref.borrow_mut();
                species.calculate_score();
                score_list.push((String::clone(species.get_name()), species.get_score()));
            }

            score_list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Equal));

            let number_to_kill: usize = std::cmp::min((self.species.len() as f64 * self.proportion_to_kill).round() as usize, self.species.len());

            //kill low bois, yeet empty spec
            for i in 0..number_to_kill {
                let name: &String = &score_list.get(i).unwrap().0;

                match self.species.get(name) {
                    None => panic!("lolwat, this shouldn't happen 231984028"),
                    Some(species_ref) => {
                        let ref_copy = Rc::clone(species_ref);
                        let mut species = ref_copy.borrow_mut();
                        species.cull(self.proportion_to_kill, &self.get_default_species());

                        if species.size() == 1 { //remove empty species
                            species.go_extinct(&self.get_default_species());
                            self.species.remove(name);
                        }
                    }
                }
            }
        }

        /*
        breed clients (to fill in clients without species)
            add the client to random species
            breed from randos in that species
            coolio
        */
        {
            //killed clients - don't have species
            let mut def_spec = self.default_species.borrow_mut();
            let clients: &mut RandomHashSet<RefCell<Client>> = def_spec.get_clients_mut();
            let mut rng = rand::thread_rng();

            //randomly add those clients to a random species
            // - replaces client's genome with one made from breeding clients in the new species
            for client in clients.get_data() {
                match self.species.values().choose(&mut rng) {
                    None => panic!("woops, no species - did we kill them all??"),
                    Some(chosen_species) => {
                        let mut species = chosen_species.borrow_mut();

                        client.borrow_mut().set_genome(Rc::new(RefCell::new(species.breed_random_clients()))); //gives client new genome
                        species.force_put(Rc::clone(client), Rc::clone(chosen_species)); // add client to the species
                    }
                }
            }

            clients.clear(); //all the clients have been moved on their end

            assert_eq!(self.get_default_species().borrow().size(), 0);
        }

        /*
            reset clients
            mutate clients
            generate new calculators

            clears all species data
        */
        {
            let mut client_names = vec![];

            //when I was directly using the name, got error cause we use &mut self in mutate_random
            for name in self.clients.keys() {
                client_names.push(String::clone(name));
            }

            for name in client_names {
                let client_ref = match self.clients.get(&name) {
                    None => continue,
                    Some(client_ref_i) => Rc::clone(client_ref_i),
                };

                let mut client = client_ref.borrow_mut();

                client.reset_client(self.get_default_species());

                //mutates genome
                let genome = client.get_genome();
                GenomeMutator::mutate_random(self, &mut genome.borrow_mut());

                client.generate_calculator(self.activation_function);
            }

            self.species.clear();
        }

        /*
        sort clients into species
        */
        self.sort_clients_into_species();
    }

    fn sort_clients_into_species(&mut self) {
        //try add to existing species
        for (_name, species_ref) in &self.species {
            let ref_for_species_borrow = Rc::clone(&species_ref);
            let mut species = ref_for_species_borrow.borrow_mut();
            for (_name, client_ref) in &self.clients {
                if species.try_add_client(Rc::clone(client_ref), Rc::clone(&species_ref),
                                          self.get_distance_constants(), self.get_species_distance_threshold()) {
                    break;
                }
            }
        }

        //add the rest of the clients to some new species
        let mut new_species: Vec<Rc<RefCell<Species>>> = Vec::new();

        'client_loop: for client_ref in self.get_default_species().borrow().get_clients().get_data() {
            //try add to the existing new species
            for species in &new_species {
                if Rc::clone(&species).borrow_mut().try_add_client(Rc::clone(client_ref), Rc::clone(species),
                                                                   self.get_distance_constants(), self.get_species_distance_threshold()) {
                    continue 'client_loop;
                }
            }

            //otherwise make new species
            let species = Species::new();
            assert_eq!(species.size(), 0);

            let species_ref = Rc::new(RefCell::new(species));
            //add client to species
            Rc::clone(&species_ref).borrow_mut().try_add_client(Rc::clone(client_ref), Rc::clone(&species_ref),
                                                                self.get_distance_constants(), self.get_species_distance_threshold());
            new_species.push(species_ref);
        }
    }

    //generate the base genome (in nodes, out nodes, 
    fn get_default_genome(&mut self) -> Genome {
        let mut genome = Genome::new();

        for i in 0..self.node_bank.len() {
            match self.get_node_by_inv_num(i) {
                None => break,
                Some(node) => {
                    if node.get_x() != 0.1 || node.get_x() != 0.9 { break; }

                    genome.add_node(node);
                }
            }
        }

        //add connections from each in node to every out node
        for in_node_inv in 0..self.num_of_input_nodes {
            for out_node_inv in self.num_of_input_nodes..self.num_of_input_nodes+self.num_of_output_nodes {
                let con_num = self.get_connection_number_from_nodes(in_node_inv, out_node_inv);
                genome.add_connection(self.new_connection(con_num, in_node_inv, out_node_inv));
            }
        }

        GenomeMutator::mutate_random(self, &mut genome);
        genome
    }

    //node0 to node1
    fn new_connection(&mut self, connection_num: usize, node0_num: usize, node1_num: usize) -> ConnectionGene {
        if let (Some(node0), Some(node1))
            = (self.node_bank.get(&node0_num), self.node_bank.get(&node1_num)) {
            //check for same nodes different con_num
            if let Some(node_exists_check_num) = self.nodes_to_connection_map.get(&(node0_num, node1_num)) {
                if node_exists_check_num != &connection_num {
                    panic!("uhhhhh! don't do that");
                }
            }

            assert!(node0.get_x() < node1.get_x());

            self.nodes_to_connection_map.insert((node0_num, node1_num), connection_num);
            return ConnectionGene::new(connection_num, Rc::clone(&node0), Rc::clone(&node1));
        }

        panic!("new_connection's nodes do not exist");
    }

    //gets from node_bank, otherwise creates new
    fn get_node_by_inv_num(&mut self, innovation_number: usize) -> Option<Rc<NodeGene>> {
        match self.node_bank.get(&innovation_number) {
            None => None,
            Some(da_node) => Some(Rc::clone(da_node))
        }
    }

    fn get_new_node_from_xy(&mut self, x: f64, y: f64) -> Rc<NodeGene> {
        let new_node = Rc::new(NodeGene::new(self.node_bank.len(), x, y));
        self.node_bank.insert(new_node.get_innovation_number(), Rc::clone(&new_node));

        new_node
    }

    fn get_replacement_for_connection(&mut self, connection: &ConnectionGene) -> (ConnectionGene, ConnectionGene, Rc<NodeGene>) {
        let replacement_node: Rc<NodeGene> = match self.connection_to_replacement_node_map.get(&connection.get_innovation_number()) {
            None => { //get new node
                let x = ( connection.to.get_x() + connection.from.get_x() ) / 2.0;
                let y = ( connection.to.get_y() + connection.from.get_y() ) / 2.0;
                self.get_new_node_from_xy(x, y)
            },
            Some(node_num) => Rc::clone(self.node_bank.get(node_num).unwrap())
        };

        let con0_num = self.get_connection_number_from_nodes(connection.from.get_innovation_number(), replacement_node.get_innovation_number());
        let con1_num = self.get_connection_number_from_nodes(replacement_node.get_innovation_number(), connection.to.get_innovation_number());

        let con0 = self.new_connection(con0_num, connection.from.get_innovation_number(), replacement_node.get_innovation_number());
        let con1 = self.new_connection(con1_num, replacement_node.get_innovation_number(), connection.to.get_innovation_number());

        (con0, con1, replacement_node)
    }

    //from node0 to node1
    fn get_connection_number_from_nodes(&mut self, node0_innovation_number: usize, node1_innovation_number: usize) -> usize {
        match self.nodes_to_connection_map.get(&(node0_innovation_number, node1_innovation_number)) {
            None => {
                self.nodes_to_connection_map.insert((node0_innovation_number, node1_innovation_number), self.nodes_to_connection_map.len());
                self.nodes_to_connection_map.len()-1
            },
            Some(con_num) => *con_num,
        }
    }

    fn get_default_species(&self) -> Rc<RefCell<Species>> {
        //because we want them all to reference the same thing
        Rc::clone(&self.default_species)
    }

    fn get_distance_constants(&self) -> (f64,f64,f64) {
        (0.1, 0.1, 0.1) //maybe move this to a constant file or something idk
        //TODO
    }

    fn get_species_distance_threshold(&self) -> f64 {
        self.species_distance_threshold
    }

    fn get_random_weight_max(&self) -> f64 {
        10.0
    }

    fn get_random_weight_shift_max(&self) -> f64 {
        50.0
    }

    fn get_max_mutation_attempts(&self) -> u64 {
        100
    }
}