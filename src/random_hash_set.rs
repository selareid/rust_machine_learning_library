use rand::Rng;
use std::rc::Rc;

pub(crate) enum HashSetRemoveTypes<T> where
    T: Eq, {
    Object(Rc<T>),
    Index(usize),
}

pub(crate) struct RandomHashSet<T> where
    T: Eq, {
    data: Vec<Rc<T>>,
}

impl<T> RandomHashSet<T> where
    T: Eq, {
    pub fn new() -> Self {
        RandomHashSet {data: Vec::new()}
    }

    pub fn contains(&self, object: &Rc<T>) -> bool {
        for x in &self.data {
            if x == object || Rc::ptr_eq(object, x) {
                return true;
            }
        }

        false
    }

    //Returns a reference to reference counter to the requested object
    pub fn random_element(&self) -> Option<&Rc<T>> {
        if self.data.len() > 0 {
            let mut rng = rand::thread_rng();
            self.data.get(rng.gen_range(0..self.data.len()))
        }
        else {
            None
        }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn push(&mut self, object: Rc<T>) -> bool {
        if !self.contains(&object) {
            self.data.push(object);
            true
        }
        else {
            false
        }
    }

    pub fn insert(&mut self, object: Rc<T>, index: usize) -> bool {
        if !self.contains(&object) {
            self.data.insert(index, object);
            true
        }
        else {
            false
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }

    //Returns a new copy of the reference counter to the requested object
    pub fn get(&self, index: usize) -> Option<Rc<T>> {
        if index >= self.data.len() {
            None
        } else {
            Some(Rc::clone(self.data.get(index).unwrap()))
        }
    }

    //return false if no value found
    pub fn remove(&mut self, to_remove: HashSetRemoveTypes<T>) -> bool {
        match to_remove {
            HashSetRemoveTypes::Index(index) => {
                if index > self.data.len() {
                    return false;
                }

                self.data.remove(index);
                true
            }
            HashSetRemoveTypes::Object(object) => {
                for index in 0..self.data.len()-1 {
                    if let Some(indexed_object) = self.data.get(index) {
                        if indexed_object == &object {
                            self.data.remove(index);
                            return true;
                        }
                    }
                }

                false
            }
        }
    }

    pub fn get_data_mut(&mut self) -> &mut Vec<Rc<T>> {
        &mut self.data
    }

    pub fn get_data(&self) -> &Vec<Rc<T>> {
        &self.data
    }
}