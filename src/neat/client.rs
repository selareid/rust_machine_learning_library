use super::species::Species;
use crate::feed_forward::calculator::Calculator;
use crate::feed_forward::genome::Genome;
use std::cell::{RefCell};
use std::rc::Rc;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

/* - Client -
Holds genome,
 species,
 calculator,
 score
 */
pub struct Client {
    name: String,
    genome: Rc<RefCell<Genome>>,
    species: Rc<RefCell<Species>>,
    score: f64,
    calculator: Option<Rc<Calculator<fn(f64)->f64>>>,
}

impl Client {
    pub(super) fn new(genome: Rc<RefCell<Genome>>, species: Rc<RefCell<Species>>) -> Self {
        let mut name: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(30)
            .map(char::from)
            .collect();
        name.insert_str(0, "client_");

        Client {
            genome,
            species,
            calculator: None,
            score: 0.0,
            name,
        }
    }

    pub(super) fn get_genome(&self) -> Rc<RefCell<Genome>> {
        Rc::clone(&self.genome)
    }

    pub(super) fn get_species(&self) -> Rc<RefCell<Species>> {
        Rc::clone(&self.species)
    }

    pub(super) fn set_species(&mut self, species: Rc<RefCell<Species>>) {
        self.species = species;
    }

    pub(super) fn use_calculator(&self, inputs: &Vec<f64>) -> Vec<f64> {
        match &self.calculator {
            None => panic!("oof, tried to use calculator on client without calculator"),
            Some(calculator) => {
                calculator.run(inputs)
            }
        }
    }

    pub(super) fn get_calculator(&self) -> Option<Rc<Calculator<fn(f64)->f64>>> {
        match &self.calculator {
            None => None,
            Some(value) => Some(Rc::clone(&value))
        }
    }

    pub(super) fn set_genome(&mut self, genome: Rc<RefCell<Genome>>) {
        self.genome = genome;
    }

    pub(super) fn generate_calculator(&mut self, activation_function: fn(f64)->f64) {
        self.calculator = Some(Rc::from(Calculator::new_from_ref(
            Rc::clone(&self.genome), activation_function
        )));
    }

    pub(super) fn get_score(&self) -> f64 {
        self.score
    }

    pub(super) fn set_score(&mut self, score: f64) {
        self.score = score;
    }

    pub(super) fn get_name(&self) -> &String {
        &self.name
    }

    //resets the client to a 'start of run' state
    pub(super) fn reset_client(&mut self, default_species: Rc<RefCell<Species>>) {
        self.score = 0.0;
        self.calculator = None;
        self.set_species(default_species);
    }
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        todo!()
    }
}

impl Eq for Client {}

//TODO make client tests