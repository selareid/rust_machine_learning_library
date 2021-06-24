pub trait Gene {
    fn get_innovation_number(&self) -> usize;

    fn set_innovation_number(&mut self, innovation_number: usize);
}
//
// pub enum InnovationNumber {
//     InnovationNumber(usize)
// }