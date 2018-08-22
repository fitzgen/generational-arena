extern crate generational_arena;
use generational_arena::Arena;
use std::collections::BTreeSet;

#[test]
fn can_get_live_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.try_insert(42).unwrap();
    assert_eq!(arena[i], 42);
}

#[test]
fn cannot_get_free_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.try_insert(42).unwrap();
    assert_eq!(arena.remove(i).unwrap(), 42);
    assert!(!arena.contains(i));
}

#[test]
fn cannot_get_other_generation_value() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.try_insert(42).unwrap();
    assert_eq!(arena.remove(i).unwrap(), 42);
    assert!(!arena.contains(i));
    let j = arena.try_insert(42).unwrap();
    assert!(!arena.contains(i));
    assert_eq!(arena[j], 42);
    assert!(i != j);
}

#[test]
fn try_insert_when_full() {
    let mut arena = Arena::with_capacity(1);
    arena.try_insert(42).unwrap();
    assert_eq!(arena.try_insert(42).unwrap_err(), 42);
}

#[test]
fn insert_many_and_cause_doubling() {
    let mut arena = Arena::new();
    let indices: Vec<_> = (0..1000).map(|i| arena.insert(i * i)).collect();
    for (i, idx) in indices.iter().cloned().enumerate() {
        assert_eq!(arena.remove(idx).unwrap(), i * i);
        assert!(!arena.contains(idx));
    }
}

#[test]
fn capacity_and_reserve() {
    let mut arena: Arena<usize> = Arena::with_capacity(42);
    assert_eq!(arena.capacity(), 42);
    arena.reserve(10);
    assert_eq!(arena.capacity(), 52);
}

#[test]
fn get_mut() {
    let mut arena = Arena::new();
    let idx = arena.insert(5);
    arena[idx] += 1;
    assert_eq!(arena[idx], 6);
}

#[test]
fn into_iter() {
    let mut arena = Arena::new();
    arena.insert(0);
    arena.insert(1);
    arena.insert(2);
    let set: BTreeSet<_> = arena.into_iter().collect();
    assert_eq!(set.len(), 3);
    assert!(set.contains(&0));
    assert!(set.contains(&1));
    assert!(set.contains(&2));
}

#[test]
#[should_panic]
fn index_deleted_item() {
    let mut arena = Arena::new();
    let idx = arena.insert(42);
    arena.remove(idx);
    arena[idx];
}

#[test]
fn out_of_bounds_index_from_other_arena() {
    let mut arena1 = Arena::new();
    let arena2 = Arena::<usize>::new();
    let idx = arena1.insert(42);
    assert!(arena2.get(idx).is_none());
}
