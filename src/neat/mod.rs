use crate::neat::species::Species;
use std::cell::{RefCell, RefMut};
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
    //distance constants
    C1: f64,
    C2: f64,
    C3: f64,
    random_weight_max: f64,
    random_weight_shift_max: f64,
    max_mutation_attempts: u64,

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
               proportion_to_kill: f64, C1: f64,
               C2: f64, C3: f64,
               random_weight_max: f64,
               random_weight_shift_max: f64,
               max_mutation_attempts: u64,) -> Self {
        let input_size = input_size+1; //for the bias node

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

            C1,
            C2,
            C3,
            random_weight_max,
            random_weight_shift_max,
            max_mutation_attempts,

            node_bank: Default::default(),
            nodes_to_connection_map: Default::default(),
            connection_to_replacement_node_map: Default::default(),

            num_of_input_nodes: input_size,
            num_of_output_nodes: output_size,
            cached_rng: rand::thread_rng(),
        };

        neat.add_nodes_for_new_neat();

        neat
    }

    fn add_nodes_for_new_neat(&mut self) {
        for i in 0..self.num_of_input_nodes {
            let y = (i / self.num_of_input_nodes) as f64;
            self.add_input_node(y);
        }

        for i in 0..self.num_of_output_nodes {
            let y = (i / self.num_of_output_nodes) as f64;
            self.add_output_node(y);
        }
    }

    fn add_output_node(&mut self, y: f64) {
        self.get_new_node_from_xy(0.9, y);
    }

    fn add_input_node(&mut self, y: f64) {
        self.get_new_node_from_xy(0.1, y);
    }

    //creates new client, adds to default species
    //returns client's id/name
    //client is ready-to-run on creation (has calculator)
    pub fn new_client(&mut self) -> String {
        //get a basic client
        let mut client = Client::new(Rc::new(RefCell::new(self.get_default_genome())),
                                 self.get_default_species(),
                                 );
        //generate calculator
        client.generate_calculator(self.activation_function);

        let name = String::clone(client.get_name());
        let client_ref = Rc::new(RefCell::new(client));

        //add client to default species
        self.put_client_in_default_species(&client_ref);

        //add client to clients
        self.clients.insert(String::clone(&name), client_ref);

        name
    }

    //run the client's calculator
    pub fn use_client(&self, client_name: &String, inputs: &Vec<f64>) -> Vec<f64> {
        let client = self.get_client_ref(client_name).borrow();
        client.use_calculator(&Neat::get_input_with_bias(inputs))
    }

    fn get_input_with_bias(inputs: &Vec<f64>) -> Vec<f64> {
        let mut inputs_with_bias: Vec<f64> = vec![1.0];
        inputs_with_bias.extend(inputs);
        inputs_with_bias
    }

    pub fn score_client(&self, client_name: &String, score: f64) {
        self.get_client_ref(client_name).borrow_mut().set_score(score);
    }

    pub fn update_clients(&mut self) {
        let mut score_list: Vec<(String, f64)> = Vec::new();
        self.evaluate_species_to_score_list(&mut score_list);
        self.cull_lowest_scored_species_and_remove_empty(&mut score_list);

        if self.species.len() > 0 {
            self.breed_clients_without_species_into_species();
        }

        self.reset_clients_mutate_and_generate_calculator();
        self.sort_clients_into_species();
    }

    fn reset_clients_mutate_and_generate_calculator(&mut self) {
        //copied for self mutable borrow reasons
        let mut client_names: Vec<String> = self.clients.keys().map(|n| String::clone(n)).collect();

        for name in client_names {
            let client_ref = Rc::clone(self.get_client_ref(&name));
            let mut client = client_ref.borrow_mut();

            self.put_client_in_default_species(&client_ref);
            client.reset_client();

            self.mutate_client_and_update_calculator(&mut client);
        }

        self.species.clear();
    }

    fn mutate_client_and_update_calculator(&mut self, client: &mut RefMut<Client>) {
        let genome = client.get_genome();
        GenomeMutator::mutate_random(self, &mut genome.borrow_mut());
        client.generate_calculator(self.activation_function);
    }

    fn put_client_in_default_species(&mut self, client_ref: &Rc<RefCell<Client>>) {
        self.get_default_species().borrow_mut().force_put(Rc::clone(&client_ref), self.get_default_species());
    }

    fn breed_clients_without_species_into_species(&mut self) {
        let default_spec_ref = self.get_default_species();
        let mut default_spec = default_spec_ref.borrow_mut();
        let default_species_clients_list: &mut RandomHashSet<RefCell<Client>> = default_spec.get_clients_mut();

        for client in default_species_clients_list.get_data() {
            self.put_client_in_random_species(client);
        }

        default_species_clients_list.clear(); //all clients have changed species

        assert_eq!(self.get_default_species().borrow().size(), 0); //TODO move this to a test
    }

    fn put_client_in_random_species(&mut self, client: &Rc<RefCell<Client>>) {
        let species_ref = self.get_random_species_ref();
        let mut species = species_ref.borrow_mut();
        species.breed_client_into_species(Rc::clone(client), Rc::clone(species_ref));
    }

    fn cull_lowest_scored_species_and_remove_empty(&mut self, score_list: &mut Vec<(String, f64)>) {
        let number_of_species_to_cull: usize = std::cmp::min((self.species.len() as f64 * self.proportion_to_kill).round() as usize, self.species.len());
        let lowest_x_scored_species_indices = 0..number_of_species_to_cull;

        for i in lowest_x_scored_species_indices {
            let name: &String = &score_list.get(i).unwrap().0;

            //reference cloned to avoid self mutable borrow error when calling if_empty_remove_species
            let species_ref = Rc::clone(self.get_species_ref(name));

            let mut species = species_ref.borrow_mut();

            species.remove_lowest_scoring_clients(self.proportion_to_kill, &self.get_default_species());
            self.if_empty_remove_species(species);
        }
    }

    fn evaluate_species_to_score_list(&mut self, score_list: &mut Vec<(String, f64)>) {
        for (_key, species_ref) in &self.species {
            let mut species = species_ref.borrow_mut();

            species.calculate_score(); //score

            score_list.push((String::clone(species.get_name()), species.get_score())); //add to score_list
        }

        score_list.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(Equal));
    }

    fn if_empty_remove_species(&mut self, mut species: RefMut<Species>) {
        let species_is_empty = species.size() <= 1;
        if species_is_empty {
            species.go_extinct_move_clients_to_default_species(&self.get_default_species());
            self.species.remove(species.get_name());
        }
    }

    fn sort_clients_into_species(&mut self) {
        self.try_add_clients_to_existing_species();
        self.add_remaining_clients_to_new_species();
    }

    fn try_add_clients_to_existing_species(&mut self) {
        for (_name, species_ref) in &self.species {
            let ref_for_species_borrow = Rc::clone(&species_ref);
            let mut species = ref_for_species_borrow.borrow_mut();

            for (_name, client_ref) in &self.clients {
                let succeeded = species.try_add_client(Rc::clone(client_ref), Rc::clone(&species_ref),
                                                       self.get_distance_constants(), self.get_species_distance_threshold());
                if succeeded { break; }
            }
        }
    }

    fn add_remaining_clients_to_new_species(&mut self) {
        let mut new_species_list: Vec<Rc<RefCell<Species>>> = Vec::new();

        for client_ref in self.get_default_species().borrow().get_clients().get_data() {
            let succeeded = self.try_add_to_existing_new_species(&mut new_species_list, client_ref);
            if succeeded { continue; }

            self.add_client_to_new_species_update_species_lists(&mut new_species_list, client_ref);
        }
    }

    fn add_client_to_new_species_update_species_lists(&mut self, new_species_list: &mut Vec<Rc<RefCell<Species>>>, client_ref: &Rc<RefCell<Client>>) {
        let species = Species::new();
        let species_ref = Rc::new(RefCell::new(species));

        Rc::clone(&species_ref).borrow_mut().try_add_client(Rc::clone(client_ref), Rc::clone(&species_ref),
                                                            self.get_distance_constants(), self.get_species_distance_threshold());

        new_species_list.push(Rc::clone(&species_ref));
        self.species.insert(String::clone(Rc::clone(&species_ref).borrow().get_name()), species_ref);
    }

    fn try_add_to_existing_new_species(&mut self, new_species: &mut Vec<Rc<RefCell<Species>>>, client_ref: &Rc<RefCell<Client>>) -> bool {
        let mut succeeded = false;

        new_species.iter().for_each(|species_ref| {
            if succeeded { return; }

            succeeded = species_ref.borrow_mut().try_add_client(Rc::clone(client_ref), Rc::clone(species_ref),
                                                                self.get_distance_constants(), self.get_species_distance_threshold());
        });

        succeeded
    }

    //generate the base genome (in nodes, out nodes,
    fn get_default_genome(&mut self) -> Genome {
        let mut genome = Genome::new();

        for i in 0..self.node_bank.len() {
            match self.get_node_by_inv_num(i) {
                None => break,
                Some(node) => {
                    if node.get_x() != 0.1 && node.get_x() != 0.9 { break; }

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

        GenomeMutator::mutate_full(self, &mut genome);
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
                let con = self.get_new_node_from_xy(x, y);
                self.connection_to_replacement_node_map.insert(connection.get_innovation_number(), con.get_innovation_number());
                con
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
        (self.C1, self.C2, self.C3) //maybe move this to a constant file or something idk
    }

    fn get_species_distance_threshold(&self) -> f64 {
        self.species_distance_threshold
    }

    fn get_random_weight_max(&self) -> f64 {
        self.random_weight_max
    }

    fn get_random_weight_shift_max(&self) -> f64 {
        self.random_weight_shift_max
    }

    fn get_max_mutation_attempts(&self) -> u64 {
        self.max_mutation_attempts
    }

    pub fn get_number_of_species(&self) -> usize {
        self.species.len()
    }

    pub fn get_number_of_clients(&self) -> usize {
        self.clients.len()
    }

    pub fn display_genome(&self, client_name: &String) {
        let client = self.get_client_ref(client_name).borrow();
        let genome_ref = client.get_genome();
        let genome = genome_ref.borrow();
        println!("{:?}", genome.connections);
    }

    fn get_client_ref(&self, client_name: &String) -> &Rc<RefCell<Client>> {
        match self.clients.get(client_name) {
            None => panic!("Illegal moment, client with name {} does not exist", client_name),
            Some(client_ref) => client_ref,
        }
    }

    fn get_species_ref(&self, species_name: &String) -> &Rc<RefCell<Species>> {
        match self.species.get(species_name) {
            None => panic!("Attempt to get species {} failed. Species does not exist", species_name),
            Some(species_ref) => species_ref,
        }
    }

    fn get_random_species_ref(&self) -> &Rc<RefCell<Species>> {
        match self.species.values().choose(&mut rand::thread_rng()) {
            None => panic!("Attempt to get random species, none found"),
            Some(species_ref) => species_ref,
        }
    }
}