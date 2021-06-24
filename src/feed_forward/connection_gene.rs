use super::gene::Gene;
use super::node_gene::NodeGene;
use std::rc::Rc;

static DEFAULT_WEIGHT: f64 = 1.0;
static DEFAULT_ENABLED: bool = true;

//Every connection is unique per genome, even if same connection
#[derive(Debug)]
pub struct ConnectionGene {
    innovation_number: usize,

    pub(crate) from: Rc<NodeGene>,
    pub(crate) to: Rc<NodeGene>,

    pub(crate) weight: f64,
    pub(crate) enabled: bool,
}

impl ConnectionGene {
    pub fn new(innovation_number: usize, from: Rc<NodeGene>, to: Rc<NodeGene>) -> Self {
        ConnectionGene{innovation_number, from, to, weight: DEFAULT_WEIGHT, enabled: DEFAULT_ENABLED}
    }

    //copies all values from given connection reference, returns new connection
    pub fn clone(old_gene: &ConnectionGene) -> Self {
        ConnectionGene{
            innovation_number: old_gene.innovation_number,
            from: Rc::clone(&old_gene.from),
            to: Rc::clone(&old_gene.to),
            weight: old_gene.weight,
            enabled: old_gene.enabled,
        }
    }
}

impl Gene for ConnectionGene {
    fn get_innovation_number(&self) -> usize {
        self.innovation_number
    }

    fn set_innovation_number(&mut self, innovation_number: usize) {
        self.innovation_number = innovation_number
    }
}

impl PartialEq for ConnectionGene {
    fn eq(&self, other: &Self) -> bool { // they gotta be equal on every single option
        self.innovation_number == other.innovation_number && self.from == other.from
            && self.to == other.to && self.weight == other.weight
            && self.enabled == other.enabled
    }
}

impl Eq for ConnectionGene {}