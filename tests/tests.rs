extern crate generational_arena;
use generational_arena::Arena;

#[test]
fn can_get_live_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.insert(42).unwrap();
    assert_eq!(*arena.get(i).unwrap(), 42);
}

#[test]
fn cannot_get_free_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.insert(42).unwrap();
    assert_eq!(arena.remove(i).unwrap(), 42);
    assert!(arena.get(i).is_none());
}

#[test]
fn cannot_get_other_generation_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.insert(42).unwrap();
    assert_eq!(arena.remove(i).unwrap(), 42);
    assert!(arena.get(i).is_none());
    let j = arena.insert(42).unwrap();
    assert!(arena.get(i).is_none());
    assert_eq!(*arena.get(j).unwrap(), 42);
    assert!(i != j);
}

#[test]
fn insert_when_full() {
    let mut arena = Arena::with_capacity(1);
    arena.insert(42).unwrap();
    assert_eq!(arena.insert(42).unwrap_err(), 42);
}
