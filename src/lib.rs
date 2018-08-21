#![no_std]
#![cfg_attr(not(feature = "std"), feature(alloc))]

#[macro_use]
extern crate cfg_if;

cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::vec::Vec;
    } else {
        extern crate alloc;
        use alloc::vec::Vec;
    }
}

use core::mem;

#[derive(Debug)]
pub struct Arena<T> {
    items: Vec<Entry<T>>,
    generation: u64,
    free_list_head: Option<usize>,
}

#[derive(Debug)]
enum Entry<T> {
    Free { next_free: Option<usize> },
    Occupied { generation: u64, value: T },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index {
    index: usize,
    generation: u64,
}

impl<T> Arena<T> {
    pub fn with_capacity(n: usize) -> Arena<T> {
        assert!(n > 0);
        let items: Vec<_> = (0..n)
            .map(|i| {
                if i == n - 1 {
                    Entry::Free { next_free: None }
                } else {
                    Entry::Free {
                        next_free: Some(i + 1),
                    }
                }
            }).collect();
        Arena {
            items,
            generation: 0,
            free_list_head: Some(0),
        }
    }

    pub fn insert(&mut self, value: T) -> Result<Index, T> {
        match self.free_list_head {
            None => Err(value),
            Some(i) => {
                match self.items[i] {
                    Entry::Occupied { .. } => panic!("corrupt free list"),
                    Entry::Free { next_free } => {
                        self.free_list_head = next_free;
                        self.items[i] = Entry::Occupied {
                            generation: self.generation,
                            value,
                        };
                        Ok(Index {
                            index: i,
                            generation: self.generation,
                        })
                    }
                }
            }
        }
    }

    pub fn remove(&mut self, i: Index) -> Option<T> {
        assert!(i.index < self.items.len());
        let entry = mem::replace(
            &mut self.items[i.index],
            Entry::Free {
                next_free: self.free_list_head,
            },
        );
        match entry {
            Entry::Occupied { generation, value } => if generation == i.generation {
                self.generation += 1;
                self.free_list_head = Some(i.index);
                Some(value)
            } else {
                self.items[i.index] = Entry::Occupied { generation, value };
                None
            },
            e @ Entry::Free { .. } => {
                self.items[i.index] = e;
                None
            }
        }
    }

    pub fn get(&self, i: Index) -> Option<&T> {
        assert!(i.index < self.items.len());
        match self.items[i.index] {
            Entry::Occupied {
                generation,
                ref value,
            }
                if generation == i.generation =>
            {
                Some(value)
            }
            _ => None,
        }
    }

    pub fn get_mut(&mut self, i: Index) -> Option<&mut T> {
        assert!(i.index < self.items.len());
        match self.items[i.index] {
            Entry::Occupied {
                generation,
                ref mut value,
            }
                if generation == i.generation =>
            {
                Some(value)
            }
            _ => None,
        }
    }
}
