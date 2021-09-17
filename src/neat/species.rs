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

#[derive(Debug)]
#[derive(Default)]
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

    // pub(super) fn try_add_client(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>, distance_constants: (f64,f64,f64), species_distance_threshold: f64) -> bool {
    //     if let Some(rep_ref) = &self.representative {
    //         if self.check_client_compatibility(&client, species_distance_threshold, distance_constants) {
    //             self.force_add_client(client, species_ref);
    //             return true;
    //         }
    //
    //         return false
    //     }
    //     else {
    //         self.force_new_client_as_rep(client, species_ref);
    //         return true;
    //     }
    // }

    pub(super) fn get_representatives_genome_rc(&self) -> Rc<RefCell<Genome>> {
        if let Some(representatives_reference) = &self.representative {
            return Rc::clone(&representatives_reference.borrow().get_genome());
        }

        panic!("uhhh, no representative. Cannot get genome for representative");
    }

    //no test written
    pub(super) fn check_genome_compatibility(&self, genome: &Genome, species_distance_threshold: f64, distance_constants: (f64, f64, f64)) -> bool {
        let clients_distance_to_rep = GenomeNeatMethods::distance(genome, &self.get_representatives_genome_rc().borrow(), distance_constants);
        clients_distance_to_rep < species_distance_threshold // below threshold
    }

    //no test written
    pub(super) fn check_client_compatibility(&self, client: &Client, species_distance_threshold: f64, distance_constants: (f64, f64, f64)) -> bool {
        self.check_genome_compatibility(&*client.get_genome().borrow(), species_distance_threshold, distance_constants)
    }

    pub(super) fn force_add_client_without_updating_clients_species(&mut self, client: Rc<RefCell<Client>>) {
        self.clients.push(client);
    }

    //adds client to species and updates client's species
    pub fn force_add_client(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>) {
        client.borrow_mut().set_species(species_ref);
        self.force_add_client_without_updating_clients_species(client);
    }

    pub(super) fn force_new_client_as_rep(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>) {
        self.representative = Some(Rc::clone(&client));
        self.force_add_client(client, species_ref);
    }

    pub(super) fn force_new_client_as_rep_without_updating_client_species(&mut self, client: Rc<RefCell<Client>>) {
        self.representative = Some(Rc::clone(&client));
        self.force_add_client_without_updating_clients_species(client);
    }

    // pub(super) fn breed_client_into_species(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>) {
    //     let new_genome_ref = Rc::new(RefCell::new(self.breed_random_clients()));
    //     client.borrow_mut().set_genome(new_genome_ref);
    //
    //     self.force_add_client(client, species_ref);
    // }

    // // MAKE SURE YOU DELETE THE SPECIES AFTER RUNNING THIS
    // pub(super) fn move_all_clients_to_default_species(&mut self, default_species: &Rc<RefCell<Species>>) {
    //     for client_ref in self.clients.get_data() {
    //         let mut client = client_ref.borrow_mut();
    //         client.set_species(Rc::clone(default_species));
    //     }
    // }

    fn calculate_fitness(&mut self) {
        let mut total_score: f64 = 0_f64;
        self.clients.get_data().iter().for_each(|client_ref| total_score += client_ref.borrow().get_score());
        self.fitness = total_score;
    }

    //assumes (non-adjusted) fitness is already calculated
    fn calculate_adjusted_fitness(&mut self) {
        self.adjusted_fitness = if self.fitness == 0.0 { 0.0 } else { self.fitness / (self.clients.size() as f64) };
    }

    pub (super) fn calculate_fitnesses(&mut self) {
        self.calculate_fitness();
        self.calculate_adjusted_fitness();
    }

    pub(super) fn get_target_population_size(&self, adjusted_population_fitness: f64) -> usize {
        (self.adjusted_fitness / adjusted_population_fitness).round() as usize
    }

    //removes all clients except one (becomes new rep)
    //also resets score to 0
    // pub(super) fn reset(&mut self, default_species: &Rc<RefCell<Species>>) {
    //     self.remove_all_clients_except_rep(default_species);
    //     self.reset_score();
    // }

    // fn reset_score(&mut self) { self.adjusted_fitness = 0.0; self.fitness = 0.0;}

    // fn remove_all_clients_except_rep(&mut self, default_species: &Rc<RefCell<Species>>) {
    //     let random_client_ref = self.get_random_client();
    //
    //     let this_species_ref = Rc::clone(&random_client_ref.borrow().get_species()); //save species reference
    //     let new_representative = Rc::clone(random_client_ref);
    //
    //     self.move_all_clients_to_default_species(default_species);
    //     self.clients.clear();
    //
    //     self.force_new_client_as_rep(new_representative, this_species_ref);
    // }

    // pub(super) fn kill_lowest_scoring_clients(&mut self, proportion_to_remove: f64) -> Vec<Rc<RefCell<Client>>> {
    //     let no_clients = self.clients.size() == 0;
    //     if no_clients { return Vec::default(); }
    //
    //     let number_to_cull: usize = std::cmp::min((self.clients.size() as f64 * proportion_to_remove).ceil() as usize, self.clients.size());
    //     self.kill_x_lowest_scoring_clients(number_to_cull)
    // }

    //Note, doesn't remove species from client
    fn kill_client_at_index_0(&mut self) -> Option<Rc<RefCell<Client>>> { //returns killed client ref
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

    //Note, no test written for this method
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

    pub(super) fn get_representative(&self) -> Option<Rc<RefCell<Client>>> {
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
        && self.fitness == other.fitness && self.representative == other.representative
        && self.clients == other.clients
    }
}

impl Eq for Species {}

#[cfg(test)]
mod species_tests {
    use super::*;

    #[test]
    fn new_species_defaults_as_expected() {
        let s: Species = Species::new();
        println!("Species name: {}", s.name);
        assert_eq!(s.fitness, 0_f64);
        assert_eq!(s.adjusted_fitness, 0_f64);
        assert_eq!(s.representative, None);
        assert_eq!(s.clients.size(), 0);
    }

    #[test]
    fn new_name_length_30_plus_species_() {
        let s: Species = Species::new();
        assert_eq!(s.name.len(), "species_".len() + 30);
    }

    #[test]
    fn get_size_returns_size() {
        let s: Species = Species::new();
        assert_eq!(s.get_size(), s.clients.size());
    }

    #[test]
    #[should_panic]
    fn getting_representative_of_new_species_panics() {
        let s: Species = Species::new();
        s.get_representatives_genome_rc();
    }

    #[test]
    fn get_representative_genome_rc_gives_genomes_reference_counter() {
        let genome = Genome::new();
        let genome_rc = Rc::new(RefCell::new(genome));

        let s: Species = Species {
            clients: RandomHashSet::new(),
            fitness: 0.0,
            adjusted_fitness: Default::default(),
            name: "".to_string(),
            representative: Some(Rc::new(RefCell::new(Client::new(Rc::clone(&genome_rc), Default::default()))))
        };

        //correct rc is eq
        assert!(Rc::ptr_eq(&genome_rc, &s.get_representatives_genome_rc()));

        //incorrect rc not eq
        let some_other_genome = Genome::new();
        let some_other_genome_rc = Rc::new(RefCell::new(some_other_genome));
        assert!(!Rc::ptr_eq(&some_other_genome_rc, &s.get_representatives_genome_rc()));
    }

    #[test]
    #[ignore]
    fn check_genome_compatibility_gives_expected_results() {
        unimplemented!();
    }

    #[test]
    #[ignore]
    fn check_client_compatibility_gives_expected_results() {
        unimplemented!();
    }

    #[test]
    fn force_add_client_without_updating_clients_species_test() {
        let mut s: Species = Species::new();

        let c_old_species_ref = Rc::new(RefCell::new(Species::new()));
        let c: Client = Client::new(Default::default(), Rc::clone(&c_old_species_ref));
        let c_ref = Rc::new(RefCell::new(c));


        s.force_add_client_without_updating_clients_species(Rc::clone(&c_ref));

        assert!(s.clients.contains(&c_ref)); //client in species as expected
        assert_eq!(s.get_size(), 1); //species size updated
        assert!(Rc::ptr_eq(&c_ref.borrow().get_species(), &c_old_species_ref)); //client's species hasn't changed
    }

    #[test]
    fn force_add_client_adds_client_and_sets_species() {
        let s_ref = Rc::new(RefCell::new(Species::new()));
        let mut s = s_ref.borrow_mut();

        let c_old_species_ref = Rc::new(RefCell::new(Species::new()));
        let c: Client = Client::new(Default::default(), Rc::clone(&c_old_species_ref));
        let c_ref = Rc::new(RefCell::new(c));


        s.force_add_client(Rc::clone(&c_ref), Rc::clone(&s_ref));

        assert!(s.clients.contains(&c_ref), "client not in species");
        assert_eq!(s.get_size(), 1); //species size updated
        assert!(Rc::ptr_eq(&c_ref.borrow().get_species(), &s_ref), "client's species wasn't updated");
    }

    #[test]
    fn force_new_client_as_rep_sets_new_client_as_representative() {
        let s_ref = Rc::new(RefCell::new(Species::new()));
        let mut s = s_ref.borrow_mut();

        let c: Client = Client::new(Default::default(), Default::default());
        let c_ref = Rc::new(RefCell::new(c));

        s.force_new_client_as_rep(Rc::clone(&c_ref), Rc::clone(&s_ref));

        let rep_rc: &Rc<RefCell<Client>> = match &s.representative { Some(rc) => rc, _ => panic!("no representative set") };

        assert!(Rc::ptr_eq(rep_rc, &c_ref), "representative is not client as expected");
    }

    #[test]
    fn force_new_client_as_rep_without_updating_client_species_sets_new_client_as_representative() {
        let s_ref = Rc::new(RefCell::new(Species::new()));
        let mut s = s_ref.borrow_mut();

        let c: Client = Client::new(Default::default(), Default::default());
        let c_ref = Rc::new(RefCell::new(c));

        s.force_new_client_as_rep_without_updating_client_species(Rc::clone(&c_ref));

        let rep_rc: &Rc<RefCell<Client>> = match &s.representative { Some(rc) => rc, _ => panic!("no representative set") };

        assert!(Rc::ptr_eq(rep_rc, &c_ref), "representative is not client as expected");
    }

    #[test]
    fn force_new_client_as_rep_without_updating_client_species_doesnt_update_clients_species() {
        let s_ref = Rc::new(RefCell::new(Species::new()));
        let mut s = s_ref.borrow_mut();

        let c: Client = Client::new(Default::default(), Default::default());
        let c_ref = Rc::new(RefCell::new(c));

        s.force_new_client_as_rep_without_updating_client_species(Rc::clone(&c_ref));

        assert!(!Rc::ptr_eq(&s_ref, &c_ref.borrow().get_species()));
    }

    #[test]
    fn calculate_fitness_0_for_new_species() {
        let mut s: Species = Species::new();
        s.calculate_fitness();
        assert_eq!(s.fitness, 0_f64);
    }

    #[test]
    fn calculate_fitness_equal_to_client_score_for_species_with_one_client() {
        let client_score: f64 = 10_f64;

        let mut s: Species = Species::new();
        let mut c: Client = Client::new(Default::default(), Default::default());
        c.set_score(client_score);
        let c_ref = Rc::new(RefCell::new(c));

        s.force_add_client_without_updating_clients_species(c_ref);
        s.calculate_fitness();
        assert_eq!(s.fitness, client_score);
    }

    #[test]
    fn calculate_fitness_equal_to_sum_of_client_scores() {
        let client_scores = [10_f64, 15_f64, 37_f64, 1000_f64, -10_f64];

        let mut s: Species = Species::new();

        for i in 0..client_scores.len() {
            let client_score = client_scores[i];
            let mut c: Client = Client::new(Default::default(), Default::default());
            c.set_score(client_score);
            let c_ref = Rc::new(RefCell::new(c));
            s.force_add_client_without_updating_clients_species(c_ref);
        }

        s.calculate_fitness();
        assert_eq!(s.fitness, client_scores.iter().sum::<f64>());
    }

    #[test]
    fn calculate_adjusted_fitness_is_0_on_new_species_after_fitness_calculated() {
        let mut s: Species = Species::new();
        s.calculate_fitness();
        s.calculate_adjusted_fitness();
        assert_eq!(s.adjusted_fitness, 0_f64);
    }

    #[test]
    fn calculate_adjusted_fitness_is_average_of_client_scores_after_fitness_calculated_when_more_than_1_client() {
        let client_scores = [10_f64, 15_f64, 37_f64, 1000_f64, -10_f64];

        let mut s: Species = Species::new();

        for i in 0..client_scores.len() {
            let client_score = client_scores[i];
            let mut c: Client = Client::new(Default::default(), Default::default());
            c.set_score(client_score);
            let c_ref = Rc::new(RefCell::new(c));
            s.force_add_client_without_updating_clients_species(c_ref);
        }

        s.calculate_fitness();
        s.calculate_adjusted_fitness();
        assert_eq!(s.adjusted_fitness, client_scores.iter().sum::<f64>() / (client_scores.len() as f64));
    }

    #[test]
    fn adjusted_fitness_0_when_fitness_0() {
        let mut s: Species = Species::new();
        s.fitness = 0_f64;
        s.calculate_adjusted_fitness();
        assert_eq!(s.adjusted_fitness, 0_f64);
    }

    #[test]
    fn calculate_fitnesses_equivalent_to_running_calculate_fitness_and_calculate_adjusted_fitness() {
        let client_scores = [10_f64, 15_f64, 37_f64, 1000_f64, -10_f64];

        let mut s1: Species = Species::new(); // calculate_fitnesses
        let mut s2: Species = Species::new(); // calculate_fitness separate
        s2.name = String::clone(&s1.name);

        for i in 0..client_scores.len() {
            let client_score = client_scores[i];
            let mut c: Client = Client::new(Default::default(), Default::default());
            c.set_score(client_score);
            let c_ref = Rc::new(RefCell::new(c));
            s1.force_add_client_without_updating_clients_species(Rc::clone(&c_ref));
            s2.force_add_client_without_updating_clients_species(c_ref);
        }

        s1.calculate_fitnesses();
        s2.calculate_fitness();
        s2.calculate_adjusted_fitness();

        assert_eq!(s1, s2);
    }

    #[test]
    fn get_target_population_size_matches_formula() {
        let mut s: Species = Species::new();
        s.adjusted_fitness = 2342_f64;
        let adjusted_population_fitness = 124_f64;
        assert_eq!(s.get_target_population_size(adjusted_population_fitness), (s.adjusted_fitness / adjusted_population_fitness).round() as usize);
    }

    #[test]
    fn kill_client_at_index_0_removes_client_at_index_0_as_expected() {
        let mut s: Species = Species::new();

        let c1: Client = Default::default();
        let c1_ref = Rc::new(RefCell::new(c1));
        let c2: Client = Default::default();
        let c2_ref = Rc::new(RefCell::new(c2));

        s.force_add_client_without_updating_clients_species(Rc::clone(&c1_ref));
        s.force_add_client_without_updating_clients_species(Rc::clone(&c2_ref));

        assert!(s.clients.contains(&c1_ref), "failed to add first client");
        assert!(s.clients.contains(&c2_ref), "failed to add second client");
        assert_eq!(s.get_size(), 2, "clients not added as expected");

        s.kill_client_at_index_0();

        assert!(!s.clients.contains(&c1_ref), "first client not removed");
        assert!(s.clients.contains(&c2_ref), "second client removed");
        assert_eq!(s.get_size(), 1);
    }

    #[test]
    fn kill_client_at_index_0_returns_client_removed() {
        let mut s: Species = Species::new();

        let c1: Client = Default::default();
        let c1_ref = Rc::new(RefCell::new(c1));
        let c2: Client = Default::default();
        let c2_ref = Rc::new(RefCell::new(c2));

        s.force_add_client_without_updating_clients_species(Rc::clone(&c1_ref));
        s.force_add_client_without_updating_clients_species(Rc::clone(&c2_ref));

        let returned_ref_opt: Option<Rc<RefCell<Client>>> = s.kill_client_at_index_0();

        match returned_ref_opt {
            Some(returned_ref) => assert!(Rc::ptr_eq(&returned_ref, &c1_ref)),
            _ => panic!("Returned Option matches 'None' which is unexpected")
        }
    }

    #[test]
    fn kill_client_at_index_0_returns_none_when_no_clients() {
        let mut s: Species = Species::new();

        assert_eq!(s.get_size(), 0);

        let returned_opt = s.kill_client_at_index_0();

        matches!(returned_opt, None);
    }

    #[test]
    fn sort_clients_by_score_least_to_greatest_sorts_clients_as_expected() {
        let client_scores: [f64; 5] = [10_f64, 15_f64, 37_f64, 1000_f64, -10_f64];
        let mut clients: Vec<Rc<RefCell<Client>>> = Vec::new();

        let mut s: Species = Species::new();

        for i in 0..client_scores.len() {
            let client_score = client_scores[i];
            let mut c: Client = Client::new(Default::default(), Default::default());
            c.set_score(client_score);
            let c_ref = Rc::new(RefCell::new(c));
            s.force_add_client_without_updating_clients_species(Rc::clone(&c_ref));
            clients.push(c_ref);
        }

        s.sort_clients_by_score_least_to_greatest();

        while s.get_size() > 0 {
            if let Some(client_at_0_r) = s.kill_client_at_index_0() {
                let mut this_client_i: usize = 0;

                for c_i in 0..clients.len() {
                    if let Some(c_r) = clients.get(c_i) {
                        if Rc::ptr_eq(&client_at_0_r, &c_r) {
                            this_client_i = c_i;
                            continue;
                        }

                        assert!(client_at_0_r.borrow().get_score() <= c_r.borrow().get_score());
                    }
                    else { panic!(); }
                }

                clients.remove(this_client_i);
            }
            else { panic!(); }
        }
    }

    #[test]
    fn kill_x_lowest_scoring_clients_kills_clients_as_expected() {
        // checks that there are x clients killed with less score that any other client remaining alive
        // after running the method

        let x_to_kill: usize = 3;
        let client_scores: [f64; 5] = [10_f64, 15_f64, 37_f64, 1000_f64, -10_f64];
        let mut clients: Vec<Rc<RefCell<Client>>> = Vec::new();

        let mut s: Species = Species::new();

        for i in 0..client_scores.len() {
            let client_score = client_scores[i];
            let mut c: Client = Client::new(Default::default(), Default::default());
            c.set_score(client_score);
            let c_ref = Rc::new(RefCell::new(c));
            s.force_add_client_without_updating_clients_species(Rc::clone(&c_ref));
            clients.push(c_ref);
        }

        s.kill_x_lowest_scoring_clients(x_to_kill);

        while s.get_size() > 0 {
            if let Some(client_at_0_r) = s.kill_client_at_index_0() {
                let mut this_client_i: usize = 0;
                let mut clients_with_less_score: usize = 0;

                for c_i in 0..clients.len() {
                    if let Some(c_r) = clients.get(c_i) {
                        if Rc::ptr_eq(&client_at_0_r, &c_r) {
                            this_client_i = c_i;
                            continue;
                        }

                        if client_at_0_r.borrow().get_score() > c_r.borrow().get_score() {
                            clients_with_less_score += 1;
                        }
                    }
                    else { panic!(); }
                }

                assert_eq!(clients_with_less_score, x_to_kill);
                clients.remove(this_client_i);
            }
            else { panic!(); }
        }
    }

    #[test]
    #[ignore]
    fn breed_random_clients_modifies_two_clients_genomes() {
        unimplemented!("I'm not sure how to test this :(");
    }

    #[test]
    fn removed_client_successfully_removes_correct_client_from_species() {
        let mut s: Species = Species::new();
        let client_ref: Rc<RefCell<Client>> = Default::default();

        //population species
        for i in 0..3 {
            s.force_add_client_without_updating_clients_species(Default::default());
        }

        s.force_add_client_without_updating_clients_species(Rc::clone(&client_ref));

        for i in 0..5 {
            let c: Client = Default::default();
            s.force_add_client_without_updating_clients_species(Default::default());
        }

        let length_before = s.get_size();
        s.remove_client(Rc::clone(&client_ref));
        assert_eq!(length_before-1, s.get_size());
        for c in s.clients.get_data() {
            assert!(!Rc::ptr_eq(c, &client_ref));
        }
    }
}