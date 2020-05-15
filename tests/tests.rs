extern crate generational_arena;
use generational_arena::Arena;
use std::collections::BTreeSet;

#[test]
fn can_decompose_index() {
    let mut arena = Arena::with_capacity(1);
    let i = arena.try_insert(42).unwrap();
    let (k, g) = i.into_raw_parts();
    let generated_i = generational_arena::Index::from_raw_parts(k, g);
    assert_eq!(arena[generated_i], 42);
}

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
fn try_insert_with_when_full() {
    let mut arena = Arena::with_capacity(1);
    let first_index = arena.try_insert_with(|_| 42).ok().unwrap();
    let returned_fn = arena.try_insert_with(|_| 42).unwrap_err();
    assert_eq!(returned_fn(first_index), 42);
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
fn insert_with_indicies_match() {
    let mut arena = Arena::new();
    let a = arena.insert_with(|idx| (40, idx));
    let b = arena.insert_with(|idx| (41, idx));
    let c = arena.insert_with(|idx| (42, idx));
    assert_eq!(arena[a].0, 40);
    assert_eq!(arena[b].0, 41);
    assert_eq!(arena[c].0, 42);
    assert_eq!(arena[a].1, a);
    assert_eq!(arena[b].1, b);
    assert_eq!(arena[c].1, c);
}

#[test]
fn try_insert_with_indicies_match() {
    let mut arena = Arena::with_capacity(3);
    let a = arena.try_insert_with(|idx| (40, idx)).ok().unwrap();
    let b = arena.try_insert_with(|idx| (41, idx)).ok().unwrap();
    let c = arena.try_insert_with(|idx| (42, idx)).ok().unwrap();
    assert_eq!(arena[a].0, 40);
    assert_eq!(arena[b].0, 41);
    assert_eq!(arena[c].0, 42);
    assert_eq!(arena[a].1, a);
    assert_eq!(arena[b].1, b);
    assert_eq!(arena[c].1, c);
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
fn get2_mut() {
    let mut arena = Arena::with_capacity(2);
    let idx1 = arena.insert(0);
    let idx2 = arena.insert(1);
    {
        let (item1, item2) = arena.get2_mut(idx1, idx2);
        assert_eq!(item1, Some(&mut 0));
        assert_eq!(item2, Some(&mut 1));
        *item1.unwrap() = 3;
        *item2.unwrap() = 4;
    }
    assert_eq!(arena[idx1], 3);
    assert_eq!(arena[idx2], 4);
}

#[test]
fn get_unknown_gen() {
    let mut arena = Arena::new();
    let idx = arena.insert(5);

    let i = idx.into_raw_parts().0;

    if let Some((el, id)) = arena.get_unknown_gen(i) {
        assert_eq!(id, idx);
        assert_eq!(*el, 5);
    } else {
        panic!("element at index {} (without generation) should exist at this point", i);
    }
    arena.remove(idx);
    if let Some((_, _)) = arena.get_unknown_gen(i) {
        panic!("element at index {} (without generation) should not exist at this point", i);
    }
}

#[test]
fn get_unknown_gen_mut() {
    let mut arena = Arena::new();
    let idx = arena.insert(5);

    let i = idx.into_raw_parts().0;

    if let Some((el, id)) = arena.get_unknown_gen_mut(i) {
        assert_eq!(id, idx);
        assert_eq!(*el, 5);
        *el += 1;
    } else {
        panic!("element at index {} (without generation) should exist at this point", i);
    }
    assert_eq!(arena.get_mut(idx).cloned(), Some(6));
    arena.remove(idx);
    if let Some((_, _)) = arena.get_unknown_gen_mut(i) {
        panic!("element at index {} (without generation) should not exist at this point", i);
    }
}

#[test]
fn get2_mut_with_same_index_but_different_generation() {
    let mut arena = Arena::with_capacity(2);
    let idx1 = arena.insert(0);
    arena.remove(idx1);
    let idx2 = arena.insert(1);
    let (item1, item2) = arena.get2_mut(idx1, idx2);
    assert_eq!(item1, None);
    assert_eq!(item2, Some(&mut 1));
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
fn out_of_bounds_get_with_index_from_other_arena() {
    let mut arena1 = Arena::with_capacity(1);
    let arena2 = Arena::<usize>::with_capacity(1);
    arena1.insert(0);
    let idx = arena1.insert(42);
    assert!(arena2.get(idx).is_none());
}

#[test]
fn out_of_bounds_remove_with_index_from_other_arena() {
    let mut arena1 = Arena::with_capacity(1);
    let mut arena2 = Arena::<usize>::with_capacity(1);
    arena1.insert(0);
    let idx = arena1.insert(42);
    assert!(arena2.remove(idx).is_none());
}

#[test]
fn out_of_bounds_get2_mut_with_index_from_other_arena() {
    let mut arena1 = Arena::with_capacity(1);
    let mut arena2 = Arena::with_capacity(2);
    let idx1 = arena1.insert(42);
    arena2.insert(0);
    let idx2 = arena2.insert(0);

    assert_eq!(arena1.get2_mut(idx1, idx2), (Some(&mut 42), None));
}

#[test]
fn drain() {
    let mut arena = Arena::new();
    let idx_1 = arena.insert("hello");
    let idx_2 = arena.insert("world");

    assert!(arena.get(idx_1).is_some());
    assert!(arena.get(idx_2).is_some());
    for (idx, value) in arena.drain() {
        assert!((idx == idx_1 && value == "hello") || (idx == idx_2 && value == "world"));
    }
    assert!(arena.get(idx_1).is_none());
    assert!(arena.get(idx_2).is_none());
}

#[test]
fn clear() {
    let mut arena = Arena::with_capacity(1);
    arena.insert(42);
    arena.insert(43);

    assert_eq!(arena.capacity(), 2);
    assert_eq!(arena.len(), 2);

    arena.clear();

    assert_eq!(arena.capacity(), 2);
    assert_eq!(arena.len(), 0);

    arena.insert(44);
    arena.insert(45);
    arena.insert(46);

    assert_eq!(arena.capacity(), 4);
    assert_eq!(arena.len(), 3);

    arena.clear();

    assert_eq!(arena.capacity(), 4);
    assert_eq!(arena.len(), 0);
}

#[test]
fn retain() {
    let mut arena = Arena::with_capacity(4);
    let index = arena.insert(2);
    arena.insert(1);
    arena.insert(4);
    arena.insert(3);

    assert_eq!(arena.len(), 4);

    arena.retain(|_, n| *n < 4);

    assert_eq!(arena.len(), 3);
    assert!(arena.iter().all(|(_, n)| *n < 4));

    arena.retain(|_, n| *n < 3);

    assert_eq!(arena.len(), 2);
    assert!(arena.iter().all(|(_, n)| *n < 3));
    assert!(arena.contains(index));

    arena.retain(|i, _| i != index);

    assert_eq!(arena.len(), 1);
    assert!(!arena.contains(index));
}
