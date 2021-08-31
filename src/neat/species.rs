/* - Species -
Holds Clients
    species' score
    species' name
    representative client
 */
use crate::neat::client::Client;
use crate::random_hash_set::{RandomHashSet, HashSetRemoveTypes};
use std::cell::RefCell;
use std::rc::Rc;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::feed_forward::genome::Genome;
use std::ops::Deref;
use crate::neat::genome_neat::GenomeNeatMethods;

pub(super) struct Species {
    clients: RandomHashSet<RefCell<Client>>,
    fitness: f64, //default 0
    adjusted_fitness: f64, //default 0
    name: String,
    representative: Option<Rc<RefCell<Client>>>
}

impl Species {
    pub(super) fn new() -> Self {
        Species {
            name: Species::generate_new_name(),
            clients: RandomHashSet::new(),
            fitness: 0_f64,
            adjusted_fitness: 0_f64,
            representative: None
        }
    }

    fn generate_new_name() -> String {
        let mut name: String = thread_rng().sample_iter(&Alphanumeric).take(30).map(char::from).collect();
        name.insert_str(0, "species_");
        name
    }

    pub(super) fn get_size(&self) -> usize {
        self.clients.size()
    }

    pub(super) fn try_add_client(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>, distance_constants: (f64,f64,f64), species_distance_threshold: f64) -> bool {
        if let Some(rep_ref) = &self.representative {
            if self.check_client_compatibility(&client, species_distance_threshold, distance_constants) {
                self.force_add_client(client, species_ref);
                return true;
            }

            return false
        }
        else {
            self.force_new_client_as_rep(client, species_ref);
            return true;
        }
    }

    pub(super) fn get_representatives_genome_rc(&self) -> Rc<RefCell<Genome>> {
        if let Some(representatives_reference) = &self.representative {
            return Rc::clone(&representatives_reference.borrow().get_genome());
        }

        panic!("uhhh, no representative. Cannot get genome for representative");
    }

    pub(super) fn check_client_compatibility(&self, client: &Rc<RefCell<Client>>, species_distance_threshold: f64, distance_constants: (f64, f64, f64)) -> bool {
        self.check_genome_compatibility(&*client.borrow().get_genome().borrow(), species_distance_threshold, distance_constants)
    }

    pub(super) fn check_genome_compatibility(&self, genome: &Genome, species_distance_threshold: f64, distance_constants: (f64, f64, f64)) -> bool {
        let clients_distance_to_rep = GenomeNeatMethods::distance(genome, &self.get_representatives_genome_rc().borrow(), distance_constants);
        clients_distance_to_rep < species_distance_threshold // below threshold
    }

    fn force_new_client_as_rep(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>) {
        self.representative = Some(Rc::clone(&client));
        self.force_add_client(client, species_ref);
    }

    //adds client to species and updates client's species
    pub fn force_add_client(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>) {
        client.borrow_mut().set_species(species_ref);
        self.force_add_client_without_updating_clients_species(client);
    }

    pub(super) fn force_add_client_without_updating_clients_species(&mut self, client: Rc<RefCell<Client>>) {
        self.clients.push(client);
    }

    pub(super) fn breed_client_into_species(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>) {
        let new_genome_ref = Rc::new(RefCell::new(self.breed_random_clients()));
        client.borrow_mut().set_genome(new_genome_ref);

        self.force_add_client(client, species_ref);
    }

    // MAKE SURE YOU DELETE THE SPECIES AFTER RUNNING THIS
    pub(super) fn move_all_clients_to_default_species(&mut self, default_species: &Rc<RefCell<Species>>) {
        for client_ref in self.clients.get_data() {
            let mut client = client_ref.borrow_mut();
            client.set_species(Rc::clone(default_species));
        }
    }

    pub (super) fn calculate_fitnesses(&mut self) {
        self.calculate_fitness();
        self.calculate_adjusted_fitness();
    }

    //assumes (non-adjusted) fitness is already calculated
    fn calculate_adjusted_fitness(&mut self) {
        self.adjusted_fitness = if self.fitness == 0.0 { 0.0 } else { self.fitness / (self.clients.size() as f64) };
    }

    fn calculate_fitness(&mut self) {
        let mut total_score: f64 = 0_f64;
        self.clients.get_data().iter().for_each(|client_ref| total_score += client_ref.borrow().get_score());
        self.fitness = total_score;
    }

    pub(super) fn get_target_population_size(&self, adjusted_population_fitness: f64) -> usize {
        (self.adjusted_fitness / adjusted_population_fitness).round() as usize
    }

    //removes all clients except one (becomes new rep)
    //also resets score to 0
    pub(super) fn reset(&mut self, default_species: &Rc<RefCell<Species>>) {
        self.remove_all_clients_except_rep(default_species);
        self.reset_score();
    }

    fn reset_score(&mut self) { self.adjusted_fitness = 0.0; self.fitness = 0.0;}

    fn remove_all_clients_except_rep(&mut self, default_species: &Rc<RefCell<Species>>) {
        let random_client_ref = self.get_random_client();

        let this_species_ref = Rc::clone(&random_client_ref.borrow().get_species()); //save species reference
        let new_representative = Rc::clone(random_client_ref);

        self.move_all_clients_to_default_species(default_species);
        self.clients.clear();

        self.force_new_client_as_rep(new_representative, this_species_ref);
    }

    pub(super) fn kill_lowest_scoring_clients(&mut self, proportion_to_remove: f64) -> Vec<Rc<RefCell<Client>>> {
        let no_clients = self.clients.size() == 0;
        if no_clients { return Vec::default(); }

        let number_to_cull: usize = std::cmp::min((self.clients.size() as f64 * proportion_to_remove).ceil() as usize, self.clients.size());
        self.kill_x_lowest_scoring_clients(number_to_cull)
    }

    pub(super) fn kill_x_lowest_scoring_clients(&mut self, number_to_remove: usize) -> Vec<Rc<RefCell<Client>>> {
        self.sort_clients_by_score_least_to_greatest();

        let mut clients_to_kill: Vec<Rc<RefCell<Client>>> = Vec::default();

        for _i in 0..number_to_remove {
            let client_ref = match self.kill_client_at_index_0() {
                Some(client_ref) => client_ref,
                None => panic!("oopsie"),
            };

            clients_to_kill.push(client_ref);
        }

        clients_to_kill
    }

    fn kill_client_at_index_0(&mut self) -> Option<Rc<RefCell<Client>>> {
        if let Some(client_ref) = self.clients.get(0) {
            self.clients.get_data_mut().remove(0); //remove client from this species
            Some(Rc::clone(&client_ref))
        }
        else {None}
    }

    fn sort_clients_by_score_least_to_greatest(&mut self) {
        self.clients.get_data_mut().sort_by(|a, b| {
            a.borrow().get_score().partial_cmp(&b.borrow().get_score()).unwrap()
        });
    }

    pub(super) fn breed_random_clients(&self) -> Genome {
        let mut random_client1_ref: Rc<RefCell<Client>> = Rc::clone(self.get_random_client());
        let mut random_client2_ref: Rc<RefCell<Client>> = Rc::clone(self.get_random_client());

        let random_client1 = random_client1_ref.borrow();
        let random_client2 = random_client2_ref.borrow();

        if random_client1.get_score() > random_client2.get_score() {
            GenomeNeatMethods::breed(random_client1.get_genome().borrow().deref(), random_client2.get_genome().borrow().deref())
        } else {
            GenomeNeatMethods::breed(random_client2.get_genome().borrow().deref(), random_client1.get_genome().borrow().deref())
        }
    }

    pub(super) fn remove_client(&mut self, client: Rc<RefCell<Client>>) {
        todo!("test this actually removes");
        self.clients.remove(HashSetRemoveTypes::Object(client));
    }

    pub(super) fn get_clients(&self) -> &RandomHashSet<RefCell<Client>> {
        &self.clients
    }

    pub(super) fn get_clients_mut(&mut self) -> &mut RandomHashSet<RefCell<Client>> {
        &mut self.clients
    }

    fn get_random_client(&self) -> &Rc<RefCell<Client>> {
        match self.clients.random_element() {
            Some(random_client_ref) => random_client_ref,
            None => panic!("no clients found :("),
        }
    }

    pub(super) fn get_fitness(&self) -> f64 {
        self.fitness
    }

    pub(super) fn get_adjusted_fitness(&self) -> f64 {
        self.adjusted_fitness
    }

    fn get_representative(&self) -> Option<Rc<RefCell<Client>>> {
        match &self.representative {
            Some(x) => Some(Rc::clone(x)), // return clone of the reference held
            None => None
        }
    }

    pub(super) fn get_name(&self) -> &String {
        &self.name
    }
}

impl PartialEq for Species {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.adjusted_fitness == other.adjusted_fitness
    }
}

impl Eq for Species {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_new_species() {
        let s: Species = Species::new();
        println!("Species name: {}", s.name);

        todo!()

        /*
        todo:
        test getneat
        test tryaddclient
        test forceput
        */
    }
}
