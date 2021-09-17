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
use rand::prelude::IteratorRandom;

#[cfg(test)]
mod genome_neat_tests;

mod client;
mod species;
mod genome_neat;

/* - Neat -
everything
 */

#[derive(Debug)]
pub struct Neat {
    species: HashMap<String, Rc<RefCell<Species>>>, //does not include default_species
    clients: HashMap<String, Rc<RefCell<Client>>>,
    adjusted_population_fitness: f64,

    activation_function: fn(f64) -> f64,
    species_distance_threshold: f64,
    proportion_to_kill: f64, //proportion to kill each run

    //mutate chances - 1 in x chance of mutating
    mutate_chance_add_node: u64,
    mutate_chance_add_connection: u64,
    mutate_chance_random_weight: u64,
    mutate_chance_weight_shift: u64,
    mutate_chance_toggle_connection: u64,
    //distance constants - c1, c2, c3
    distance_constants: (f64, f64, f64),
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
               proportion_to_kill: f64, c1: f64,
               c2: f64, c3: f64,
               random_weight_max: f64,
               random_weight_shift_max: f64,
               max_mutation_attempts: u64,) -> Self {
        let input_size = input_size+1; //for the bias node

        let mut neat = Neat {
            species: Default::default(),
            clients: Default::default(),
            adjusted_population_fitness: 0_f64,

            activation_function,
            species_distance_threshold,
            proportion_to_kill,
            mutate_chance_add_node,
            mutate_chance_add_connection,
            mutate_chance_random_weight,
            mutate_chance_weight_shift,
            mutate_chance_toggle_connection,

            distance_constants: (c1, c2, c3),
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
        self.create_new_node_from_xy(0.9, y);
    }

    fn add_input_node(&mut self, y: f64) {
        self.create_new_node_from_xy(0.1, y);
    }

    //creates new client, adds to default species
    //returns client's id/name
    //client is ready-to-run on creation (has calculator)
    pub fn new_client(&mut self) -> String {
        
        let clients_genome: Genome = self.generate_new_genome();
        let clients_species: Rc<RefCell<Species>> = self.get_species_for_genome(&clients_genome);
        let mut client: Client = Client::new(Rc::from(RefCell::from(clients_genome)), Rc::clone(&clients_species));

        client.generate_calculator(self.activation_function);

        let name = String::clone(client.get_name());
        let client_ref = Rc::new(RefCell::new(client));

        //we don't update client cause already set
        let mut c_species = clients_species.borrow_mut();
        if let None = c_species.get_representative() { //set as rep if no rep
            c_species.force_new_client_as_rep_without_updating_client_species(Rc::clone(&client_ref));
        }
        else {
            c_species.force_add_client_without_updating_clients_species(Rc::clone(&client_ref));
        }

        self.clients.insert(String::clone(&name), client_ref); //add client to clients

        name
    }

    //run the client's calculator
    pub fn use_client(&self, client_name: &String, inputs: &Vec<f64>) -> Vec<f64> {
        let client = self.get_client_ref(client_name).borrow();
        client.use_calculator(&Neat::add_bias_to_inputs(inputs))
    }

    fn add_bias_to_inputs(inputs: &Vec<f64>) -> Vec<f64> {
        let mut inputs_with_bias: Vec<f64> = vec![1.0];
        inputs_with_bias.extend(inputs);
        inputs_with_bias
    }

    pub fn score_client(&self, client_name: &String, score: f64) {
        self.get_client_ref(client_name).borrow_mut().set_score(score);
    }

    pub fn update_clients(&mut self) {
        self.evaluate_population();
        self.adjust_species_populations();
        self.mutate_genomes_and_recheck_species();
    }

    fn mutate_genomes_and_recheck_species(&mut self) {
        let client_rcs: Vec<Rc<RefCell<Client>>> = self.clients.iter().map(|(_k, client_ref)| Rc::clone(client_ref)).collect();

        for client_ref in client_rcs {
            self.mutate_client_and_update_calculator(&client_ref);
            self.give_client_new_genome_from_species_breeding();
            self.ensure_client_in_correct_species(&client_ref)
        }
    }

    fn mutate_client_and_update_calculator(&mut self, client_ref: &Rc<RefCell<Client>>) {
        let mut client: RefMut<Client> = client_ref.borrow_mut();
        let genome = client.get_genome();
        GenomeMutator::mutate_random(self, &mut genome.borrow_mut());
        client.generate_calculator(self.activation_function);
    }

    fn give_client_new_genome_from_species_breeding(&mut self) {
        // todo!()
    }

    fn ensure_client_in_correct_species(&mut self, client_ref: &Rc<RefCell<Client>>) {
        //=====Recheck Species=====
        //is client in correct species
        //if yes, continue
        //if no, put in correct species

        let mut client: RefMut<Client> = client_ref.borrow_mut();
        let client_species_ref = client.get_species();

        let mut client_species_mut = client_species_ref.borrow_mut();

        let is_representative_of_species = match &client_species_mut.get_representative() {
            Some(rep_ref) => Rc::ptr_eq(rep_ref, &client_ref),
            None => false,
        };

        let compatible_with_current_species = is_representative_of_species || client_species_mut.check_client_compatibility(&client, self.species_distance_threshold, self.distance_constants);

        if !compatible_with_current_species {
            //remove from current species
            client_species_mut.remove_client(Rc::clone(&client_ref));
            drop(client_species_mut);

            //add client to some other species
            let chosen_species_ref = self.get_species_for_genome(&client.get_genome().borrow());
            client.set_species(Rc::clone(&chosen_species_ref));

            let mut chosen_species = chosen_species_ref.borrow_mut();
            if let None = chosen_species.get_representative() { //if no rep (new species) set as rep
                chosen_species.force_new_client_as_rep_without_updating_client_species(Rc::clone(&client_ref));
            } else {
                chosen_species.force_add_client_without_updating_clients_species(Rc::clone(&client_ref));
            }
        }
    }

    fn evaluate_population(&mut self) { //evaluates population as a whole and each species individually
        let mut sum_of_client_fitness = 0_f64;

        for (_key, species_ref) in &self.species {
            let mut species = species_ref.borrow_mut();

            species.calculate_fitnesses();

            sum_of_client_fitness += species.get_fitness();
        }

        self.adjusted_population_fitness = sum_of_client_fitness / self.clients.len() as f64;
    }

    fn adjust_species_populations(&mut self) {
        let species_refs: Vec<Rc<RefCell<Species>>> = self.species.iter().map(|(_k, r)| Rc::clone(r)).collect(); //to separate from mutable self

        for species_ref in species_refs {
            let mut species = species_ref.borrow_mut();
            let species_size = species.get_size();
            let target_population = species.get_target_population_size(self.adjusted_population_fitness);

            if species_size > target_population {
                let lowest_scoring_clients: Vec<Rc<RefCell<Client>>> = species.kill_x_lowest_scoring_clients(species_size-target_population);
                self.kill_clients(lowest_scoring_clients)
            } else if species_size < target_population {
                drop(species);
                self.create_and_add_x_new_clients_for_species(target_population-species_size, Rc::clone(&species_ref));
            }
        }
    }

    fn kill_clients(&mut self, clients_to_kill: Vec<Rc<RefCell<Client>>>) {
        for client_ref in clients_to_kill {
            let client = client_ref.borrow();
            self.clients.remove(client.get_name());
        }
    }

    //generate the base genome (in nodes, out nodes,
    fn generate_new_genome(&mut self) -> Genome {
        let mut genome = Genome::new();

        for i in 0..self.node_bank.len() {
            match self.find_node_from_innovation_number(i) {
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
                let con_num = self.find_connection_num_from_its_nodes(in_node_inv, out_node_inv);
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
    fn find_node_from_innovation_number(&mut self, innovation_number: usize) -> Option<Rc<NodeGene>> {
        match self.node_bank.get(&innovation_number) {
            None => None,
            Some(da_node) => Some(Rc::clone(da_node))
        }
    }

    fn create_new_node_from_xy(&mut self, x: f64, y: f64) -> Rc<NodeGene> {
        let new_node = Rc::new(NodeGene::new(self.node_bank.len(), x, y));
        self.node_bank.insert(new_node.get_innovation_number(), Rc::clone(&new_node));

        new_node
    }

    fn find_replacement_data_for_connection(&mut self, connection: &ConnectionGene) -> (ConnectionGene, ConnectionGene, Rc<NodeGene>) {
        let replacement_node: Rc<NodeGene> = match self.connection_to_replacement_node_map.get(&connection.get_innovation_number()) {
            None => { //get new node
                let x = ( connection.to.get_x() + connection.from.get_x() ) / 2.0;
                let y = ( connection.to.get_y() + connection.from.get_y() ) / 2.0;
                let con = self.create_new_node_from_xy(x, y);
                self.connection_to_replacement_node_map.insert(connection.get_innovation_number(), con.get_innovation_number());
                con
            },
            Some(node_num) => Rc::clone(self.node_bank.get(node_num).unwrap())
        };

        let con0_num = self.find_connection_num_from_its_nodes(connection.from.get_innovation_number(), replacement_node.get_innovation_number());
        let con1_num = self.find_connection_num_from_its_nodes(replacement_node.get_innovation_number(), connection.to.get_innovation_number());

        let con0 = self.new_connection(con0_num, connection.from.get_innovation_number(), replacement_node.get_innovation_number());
        let con1 = self.new_connection(con1_num, replacement_node.get_innovation_number(), connection.to.get_innovation_number());

        (con0, con1, replacement_node)
    }

    //from node0 to node1
    fn find_connection_num_from_its_nodes(&mut self, node0_innovation_number: usize, node1_innovation_number: usize) -> usize {
        match self.nodes_to_connection_map.get(&(node0_innovation_number, node1_innovation_number)) {
            None => {
                self.nodes_to_connection_map.insert((node0_innovation_number, node1_innovation_number), self.nodes_to_connection_map.len());
                self.nodes_to_connection_map.len()-1
            },
            Some(con_num) => *con_num,
        }
    }

    pub fn get_number_of_species(&self) -> usize {
        self.species.len()
    }

    pub fn get_number_of_clients(&self) -> usize {
        self.clients.len()
    }

    pub fn get_client_names(&self) -> Vec<String> {
        self.clients.keys().map(|n| String::clone(n)).collect::<Vec<String>>()
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

    fn create_and_add_x_new_clients_for_species(&mut self, number_of_new_clients: usize, species_ref: Rc<RefCell<Species>>) {
        let mut species = species_ref.borrow_mut();

        for _i in 0..number_of_new_clients {
            self.new_client_via_species_breeding(&species_ref, &mut species);
        }
    }

    fn new_client_via_species_breeding(&mut self, species_ref: &Rc<RefCell<Species>>, species: &mut RefMut<Species>) -> Option<Rc<RefCell<Client>>> {
        let new_clients_genome = Rc::from(RefCell::from(species.breed_random_clients()));
        let new_client = Client::new(new_clients_genome, Rc::clone(&species_ref));
        let client_name = String::clone(new_client.get_name());
        let client_ref = Rc::from(RefCell::from(new_client));
        species.force_add_client_without_updating_clients_species(Rc::clone(&client_ref));
        self.clients.insert(client_name, client_ref)
    }

    fn get_species_for_genome(&mut self, genome: &Genome) -> Rc<RefCell<Species>> {
        match self.try_find_existing_species_for_genome(genome) {
            Some(species_ref) => species_ref,
            None => self.get_new_species_added_to_species()
        }
    }

    fn try_find_existing_species_for_genome(&mut self, genome: &Genome) -> Option<Rc<RefCell<Species>>> {
        for (_name, species_ref) in self.species.iter() {
            let species = species_ref.borrow();
            if species.check_genome_compatibility(genome, self.species_distance_threshold, self.distance_constants) {
                return Some(Rc::clone(&species_ref))
            }
        }

        None //no species found
    }

    fn get_new_species_added_to_species(&mut self) -> Rc<RefCell<Species>> {
        //create new species
        let species = Species::new();
        let species_name = String::clone(species.get_name());
        let species_ref = Rc::new(RefCell::new(species));

        //add to self.species
        self.species.insert(species_name, Rc::clone(&species_ref));

        species_ref
    }
}

#[cfg(test)]
mod neat_tests {
    use super::*;
    use crate::activation_functions::ActivationFunctions;

    fn default_neat() -> Neat {
        Neat {
            species: Default::default(),
            clients: Default::default(),
            adjusted_population_fitness: 0.0,
            activation_function: ActivationFunctions::identity,
            species_distance_threshold: 0.0,
            proportion_to_kill: 0.0,
            mutate_chance_add_node: 0,
            mutate_chance_add_connection: 0,
            mutate_chance_random_weight: 0,
            mutate_chance_weight_shift: 0,
            mutate_chance_toggle_connection: 0,
            distance_constants: (0.0, 0.0, 0.0),
            random_weight_max: 0.0,
            random_weight_shift_max: 0.0,
            max_mutation_attempts: 0,
            node_bank: Default::default(),
            nodes_to_connection_map: Default::default(),
            connection_to_replacement_node_map: Default::default(),
            num_of_input_nodes: 0,
            num_of_output_nodes: 0,
            cached_rng: Default::default()
        }
    }

    #[test]
    fn get_client_names_returns_vector_with_names_of_all_clients() {
        let mut n = default_neat();
        println!("{}", n.species.len());
        let mut names: Vec<String> = Default::default();

        for i in 0..50 {
            names.push(n.new_client());
        }

        let returned_names = n.get_client_names();

        //check equivalence
        assert!(returned_names.iter().all(|n| names.contains(n)));
        assert!(names.iter().all(|n| returned_names.contains(n)));
    }
}