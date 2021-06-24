use super::gene;

//Nodes as used in a genome
//Nodes don't really hold unique value (for two nodes in the same position) so we can reference the same node from everywhere
#[derive(Debug)]
pub struct NodeGene {
    innovation_number: usize,
    x: f64, // 0.1 is input, 0.9 is output, everything is in between
    y: f64,
}

impl NodeGene {
    pub fn new(innovation_number: usize, x: f64, y: f64) -> Self {
        NodeGene {innovation_number, x, y}
    }

    pub fn get_x(&self) -> f64 {
        self.x
    }

    pub fn get_y(&self) -> f64 {
        self.y
    }
}

impl gene::Gene for NodeGene {
    fn get_innovation_number(&self) -> usize {
        self.innovation_number
    }

    fn set_innovation_number(&mut self, innovation_number: usize) {
        self.innovation_number = innovation_number;
    }
}

impl PartialEq for NodeGene {
    fn eq(&self, other: &Self) -> bool {
        self.innovation_number == other.innovation_number && self.x == other.x && self.y == other.y
    }
}

impl Eq for NodeGene {}