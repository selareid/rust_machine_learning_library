use crate::random_hash_set::*;
use std::rc::Rc;
use std::cell::RefCell;

#[test]
fn test_random_hash_set() {
    let mut my_hash_set: RandomHashSet<String> = RandomHashSet::new();

    let my_string1: Rc<String> = Rc::new(String::from("string 1 this is uwu"));
    let my_string2: Rc<String> = Rc::new(String::from("string 2 this is uwu"));
    let my_string3: Rc<String> = Rc::new(String::from("string 3 this is uwu"));

    my_hash_set.push(Rc::clone(&my_string1));
    my_hash_set.push(Rc::clone(&my_string2));

    //tests get
    assert_eq!(my_hash_set.get(0).unwrap(), my_string1, "testing get");
    assert_eq!(my_hash_set.get(1).unwrap(), my_string2, "testing get");
    assert_eq!(my_hash_set.get(11), None, "testing get, bad index");

    //tests contains
    assert!(my_hash_set.contains(&my_string2), "testing contains");
    assert!(!my_hash_set.contains(&my_string3), "testing contains when it doesn't");

    //tests random element
    let random_element = Rc::clone(my_hash_set.random_element().unwrap());
    // println!("{:?}", random_element);
    assert!(random_element == my_string1 || random_element == my_string2, "testing random element");

    //tests push
    my_hash_set.push(Rc::new(String::from("string 44 this is uwu")));

    assert_eq!(my_hash_set.size(), 3, "checking size");

    //testing remove
    assert!(my_hash_set.remove(HashSetRemoveTypes::Index(0)), "testing remove by index");
    assert!(!my_hash_set.remove(HashSetRemoveTypes::Index(12)), "testing remove by bad index");
    assert_eq!(my_hash_set.size(), 2, "checking remove worked");

    assert!(my_hash_set.remove(HashSetRemoveTypes::Object(my_string2)), "testing remove by object");
    assert!(!my_hash_set.remove(HashSetRemoveTypes::Object(my_string1)), "testing remove by bad object");
    assert_eq!(my_hash_set.size(), 1, "checking removes worked");

    //tests clear
    my_hash_set.clear();
    assert_eq!(my_hash_set.size(), 0, "checking clear");

    //try get data
    assert_eq!(my_hash_set.get_data().len(), 0, "checking get data and that the data len is 0");

    //tests random
    assert_eq!(my_hash_set.random_element(), None, "testing random element with empty set");

    //tests contains when false
    assert!(!my_hash_set.contains(&my_string3));
}

//text with cells for mutability
#[test]
fn test_ref_cell_in_random_hash_set() {
    let mut my_hash_set: RandomHashSet<RefCell<String>> = RandomHashSet::new();

    let my_mutable_reference: Rc<RefCell<_>> = Rc::new(RefCell::new(String::from("this is mutable string 1 uwu uwu")));
    let ref_2: Rc<RefCell<String>> = Rc::clone(&my_mutable_reference);

    my_hash_set.push(Rc::clone(&my_mutable_reference));
    my_hash_set.push(Rc::clone(&ref_2));

    {
        *ref_2.borrow_mut() = String::from("yolo changed the string lol");
    }

    {
        assert_eq!(*my_mutable_reference.borrow(), "yolo changed the string lol");

    }

    {
        *my_mutable_reference.borrow_mut() = String::from("changed it a second time uwu");
    }

    assert_eq!(*ref_2.borrow(), "changed it a second time uwu");
}