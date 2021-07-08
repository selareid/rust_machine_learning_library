/* - Species -
Holds Clients
    species' score
    species' name
    representative client
 */
use crate::neat::client::Client;
use crate::random_hash_set::RandomHashSet;
use std::cell::RefCell;
use std::rc::Rc;
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;
use crate::feed_forward::genome::Genome;
use std::ops::Deref;
use crate::neat::genome_neat::GenomeNeatMethods;

pub(super) struct Species {
    clients: RandomHashSet<RefCell<Client>>,
    score: f64, //default 0
    name: String,
    representative: Option<Rc<RefCell<Client>>>
}

impl Species {
    pub(super) fn new() -> Self {
        let mut name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        name.insert_str(0, "species_");

        Species {
            name,
            clients: RandomHashSet::new(),
            score: 0.0,
            representative: None
        }
    }

    pub(super) fn size(&self) -> usize {
        self.clients.size()
    }

    pub(super) fn try_add_client(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>, distance_constants: (f64,f64,f64), species_distance_threshold: f64) -> bool {
        if let Some(rep_ref) = &self.representative {
            if GenomeNeatMethods::distance(&*client.borrow().get_genome().borrow(), &*rep_ref.borrow().get_genome().borrow(), distance_constants) < species_distance_threshold {
                self.force_put(client, species_ref);
                return true;
            }
        }
        else { // if no representative, this client becomes rep
            self.representative = Some(Rc::clone(&client));
            self.force_put(client, species_ref);
            return true;
        }
        false
    }

    //adds client to species and updates client's species
    pub(super) fn force_put(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>) {
        client.borrow_mut().set_species(species_ref);
        self.clients.push(client);
    }

    pub(super) fn breed_client_into_species(&mut self, client: Rc<RefCell<Client>>, species_ref: Rc<RefCell<Species>>) {
        let new_genome_ref = Rc::new(RefCell::new(self.breed_random_clients()));
        client.borrow_mut().set_genome(new_genome_ref);

        self.force_put(client, species_ref);
    }

    // MAKE SURE YOU DELETE THE SPECIES AFTER RUNNING THIS
    pub(super) fn go_extinct_move_clients_to_default_species(&mut self, default_species: &Rc<RefCell<Species>>) {
        for client_ref in self.clients.get_data() {
            let mut client = client_ref.borrow_mut();
            client.set_species(Rc::clone(default_species));
        }
    }

    pub (super) fn calculate_score(&mut self) {
        let mut total_score: f64 = 0.0;

        for client_ref in self.clients.get_data() {
            let client = client_ref.borrow();
            total_score += client.get_score();
        }

        if total_score == 0.0 {
            self.score = 0.0;
        } else {
            self.score = total_score / (self.clients.size() as f64);
        }
    }

    //removes all clients except one (becomes new rep)
    //also resets score to 0
    pub(super) fn reset(&mut self, default_species: &Rc<RefCell<Species>>) {
        if let Some(random_client_ref) = self.clients.random_element() {
            let this_species_ref = Rc::clone(&random_client_ref.borrow().get_species()); //save species reference

            self.representative = Some(Rc::clone(random_client_ref));

            for client_ref in self.clients.get_data() {
                client_ref.borrow_mut().set_species(Rc::clone(default_species)); //reset clients' species
            }


            self.clients.clear();

            if let Some(rep_ref) = &self.representative {
                self.clients.push(Rc::clone(rep_ref)); //add rep back to clients
                rep_ref.borrow_mut().set_species(this_species_ref); //set rep's species
            }
        }

        self.score = 0.0; //reset score
    }

    pub(super) fn remove_lowest_scoring_clients(&mut self, proportion_to_remove: f64, default_species: &Rc<RefCell<Species>>) {
        if self.clients.size() == 0 {
            return;
        }

        //sort clients by score, least to greatest
        self.clients.get_data_mut().sort_by(|a, b| {
            a.borrow().get_score().partial_cmp(&b.borrow().get_score()).unwrap()
        });

        let number_to_cull: usize = std::cmp::min((self.clients.size() as f64 * proportion_to_remove).ceil() as usize, self.clients.size());

        //remove first x (number to cull) clients
        for i in 0..number_to_cull {
            if let Some(client_ref) = self.clients.get(0) {
                self.clients.get_data_mut().remove(0); //remove client from this species

                //reset client's species
                Rc::clone(default_species).borrow_mut().force_put(client_ref, Rc::clone(default_species));
            }
        }
    }

    pub(super) fn breed_random_clients(&self) -> Genome {
        let mut random_client1_ref: Rc<RefCell<Client>>;
        let mut random_client2_ref: Rc<RefCell<Client>>;

        if let Some(ran_1) = self.clients.random_element() {
            random_client1_ref = Rc::clone(ran_1);
        } else {
            panic!("Didn't get element");
        }

        if let Some(ran_2) = self.clients.random_element() {
            random_client2_ref = Rc::clone(ran_2);
        } else {
            panic!("Didn't get element");
        }

        let random_client1 = random_client1_ref.borrow();
        let random_client2 = random_client2_ref.borrow();

        if random_client1.get_score() > random_client2.get_score() {
            GenomeNeatMethods::breed(random_client1.get_genome().borrow().deref(), random_client2.get_genome().borrow().deref())
        } else {
            GenomeNeatMethods::breed(random_client2.get_genome().borrow().deref(), random_client1.get_genome().borrow().deref())
        }
    }

    pub(super) fn get_clients(&self) -> &RandomHashSet<RefCell<Client>> {
        &self.clients
    }

    pub(super) fn get_clients_mut(&mut self) -> &mut RandomHashSet<RefCell<Client>> {
        &mut self.clients
    }

    pub(super) fn get_score(&self) -> f64 {
        self.score
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
        self.name == other.name && self.score == other.score
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
