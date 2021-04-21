/*!
[![](https://docs.rs/generational-arena/badge.svg)](https://docs.rs/generational-arena/)
[![](https://img.shields.io/crates/v/generational-arena.svg)](https://crates.io/crates/generational-arena)
[![](https://img.shields.io/crates/d/generational-arena.svg)](https://crates.io/crates/generational-arena)
[![Travis CI Build Status](https://travis-ci.org/fitzgen/generational-arena.svg?branch=master)](https://travis-ci.org/fitzgen/generational-arena)

A safe arena allocator that allows deletion without suffering from [the ABA
problem](https://en.wikipedia.org/wiki/ABA_problem) by using generational
indices.

Inspired by [Catherine West's closing keynote at RustConf
2018](https://www.youtube.com/watch?v=aKLntZcp27M), where these ideas (and many
more!) were presented in the context of an Entity-Component-System for games
programming.

## What? Why?

Imagine you are working with a graph and you want to add and delete individual
nodes at a time, or you are writing a game and its world consists of many
inter-referencing objects with dynamic lifetimes that depend on user
input. These are situations where matching Rust's ownership and lifetime rules
can get tricky.

It doesn't make sense to use shared ownership with interior mutability (i.e.
`Rc<RefCell<T>>` or `Arc<Mutex<T>>`) nor borrowed references (ie `&'a T` or `&'a
mut T`) for structures. The cycles rule out reference counted types, and the
required shared mutability rules out borrows. Furthermore, lifetimes are dynamic
and don't follow the borrowed-data-outlives-the-borrower discipline.

In these situations, it is tempting to store objects in a `Vec<T>` and have them
reference each other via their indices. No more borrow checker or ownership
problems! Often, this solution is good enough.

However, now we can't delete individual items from that `Vec<T>` when we no
longer need them, because we end up either

* messing up the indices of every element that follows the deleted one, or

* suffering from the [ABA
  problem](https://en.wikipedia.org/wiki/ABA_problem). To elaborate further, if
  we tried to replace the `Vec<T>` with a `Vec<Option<T>>`, and delete an
  element by setting it to `None`, then we create the possibility for this buggy
  sequence:

    * `obj1` references `obj2` at index `i`

    * someone else deletes `obj2` from index `i`, setting that element to `None`

    * a third thing allocates `obj3`, which ends up at index `i`, because the
      element at that index is `None` and therefore available for allocation

    * `obj1` attempts to get `obj2` at index `i`, but incorrectly is given
      `obj3`, when instead the get should fail.

By introducing a monotonically increasing generation counter to the collection,
associating each element in the collection with the generation when it was
inserted, and getting elements from the collection with the *pair* of index and
the generation at the time when the element was inserted, then we can solve the
aforementioned ABA problem. When indexing into the collection, if the index
pair's generation does not match the generation of the element at that index,
then the operation fails.

## Features

* Zero `unsafe`
* Well tested, including quickchecks
* `no_std` compatibility
* All the trait implementations you expect: `IntoIterator`, `FromIterator`,
  `Extend`, etc...

## Usage

First, add `generational-arena` to your `Cargo.toml`:

```toml
[dependencies]
generational-arena = "0.2"
```

Then, import the crate and use the
[`generational_arena::Arena`](./struct.Arena.html) type!

```rust
extern crate generational_arena;
use generational_arena::Arena;

let mut arena = Arena::new();

// Insert some elements into the arena.
let rza = arena.insert("Robert Fitzgerald Diggs");
let gza = arena.insert("Gary Grice");
let bill = arena.insert("Bill Gates");

// Inserted elements can be accessed infallibly via indexing (and missing
// entries will panic).
assert_eq!(arena[rza], "Robert Fitzgerald Diggs");

// Alternatively, the `get` and `get_mut` methods provide fallible lookup.
if let Some(genius) = arena.get(gza) {
    println!("The gza gza genius: {}", genius);
}
if let Some(val) = arena.get_mut(bill) {
    *val = "Bill Gates doesn't belong in this set...";
}

// We can remove elements.
arena.remove(bill);

// Insert a new one.
let murray = arena.insert("Bill Murray");

// The arena does not contain `bill` anymore, but it does contain `murray`, even
// though they are almost certainly at the same index within the arena in
// practice. Ambiguities are resolved with an associated generation tag.
assert!(!arena.contains(bill));
assert!(arena.contains(murray));

// Iterate over everything inside the arena.
for (idx, value) in &arena {
    println!("{:?} is at {:?}", value, idx);
}
```

## `no_std`

To enable `no_std` compatibility, disable the on-by-default "std" feature.

```toml
[dependencies]
generational-arena = { version = "0.2", default-features = false }
```

### Serialization and Deserialization with [`serde`](https://crates.io/crates/serde)

To enable serialization/deserialization support, enable the "serde" feature.

```toml
[dependencies]
generational-arena = { version = "0.2", features = ["serde"] }
```
 */

#![forbid(unsafe_code, missing_docs, missing_debug_implementations)]
#![no_std]

cfg_if::cfg_if! {
    if #[cfg(feature = "std")] {
        extern crate std;
        use std::vec::{self, Vec};
    } else {
        extern crate alloc;
        use alloc::vec::{self, Vec};
    }
}

use core::cmp;
use core::iter::{self, Extend, FromIterator, FusedIterator};
use core::mem;
use core::ops;
use core::slice;

#[cfg(feature = "serde")]
mod serde_impl;

/// The `Arena` allows inserting and removing elements that are referred to by
/// `Index`.
///
/// [See the module-level documentation for example usage and motivation.](./index.html)
#[derive(Clone, Debug)]
pub struct Arena<T> {
    items: Vec<Entry<T>>,
    generation: u64,
    free_list_head: Option<usize>,
    len: usize,
}

#[derive(Clone, Debug)]
enum Entry<T> {
    Free { next_free: Option<usize> },
    Occupied { generation: u64, value: T },
}

/// An index (and generation) into an `Arena`.
///
/// To get an `Index`, insert an element into an `Arena`, and the `Index` for
/// that element will be returned.
///
/// # Examples
///
/// ```
/// use generational_arena::Arena;
///
/// let mut arena = Arena::new();
/// let idx = arena.insert(123);
/// assert_eq!(arena[idx], 123);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Index {
    index: usize,
    generation: u64,
}

impl Index {
    /// Create a new `Index` from its raw parts.
    ///
    /// The parts must have been returned from an earlier call to
    /// `into_raw_parts`.
    ///
    /// Providing arbitrary values will lead to malformed indices and ultimately
    /// panics.
    pub fn from_raw_parts(a: usize, b: u64) -> Index {
        Index {
            index: a,
            generation: b,
        }
    }

    /// Convert this `Index` into its raw parts.
    ///
    /// This niche method is useful for converting an `Index` into another
    /// identifier type. Usually, you should prefer a newtype wrapper around
    /// `Index` like `pub struct MyIdentifier(Index);`.  However, for external
    /// types whose definition you can't customize, but which you can construct
    /// instances of, this method can be useful.
    pub fn into_raw_parts(self) -> (usize, u64) {
        (self.index, self.generation)
    }
}

const DEFAULT_CAPACITY: usize = 4;

impl<T> Default for Arena<T> {
    fn default() -> Arena<T> {
        Arena::new()
    }
}

impl<T> Arena<T> {
    /// Constructs a new, empty `Arena`.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::<usize>::new();
    /// # let _ = arena;
    /// ```
    pub fn new() -> Arena<T> {
        Arena::with_capacity(DEFAULT_CAPACITY)
    }

    /// Constructs a new, empty `Arena<T>` with the specified capacity.
    ///
    /// The `Arena<T>` will be able to hold `n` elements without further allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::with_capacity(10);
    ///
    /// // These insertions will not require further allocation.
    /// for i in 0..10 {
    ///     assert!(arena.try_insert(i).is_ok());
    /// }
    ///
    /// // But now we are at capacity, and there is no more room.
    /// assert!(arena.try_insert(99).is_err());
    /// ```
    pub fn with_capacity(n: usize) -> Arena<T> {
        let n = cmp::max(n, 1);
        let mut arena = Arena {
            items: Vec::new(),
            generation: 0,
            free_list_head: None,
            len: 0,
        };
        arena.reserve(n);
        arena
    }

    /// Clear all the items inside the arena, but keep its allocation.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::with_capacity(1);
    /// arena.insert(42);
    /// arena.insert(43);
    ///
    /// arena.clear();
    ///
    /// assert_eq!(arena.capacity(), 2);
    /// ```
    pub fn clear(&mut self) {
        self.items.clear();
        self.len = 0;
        self.reserve(self.items.capacity());
    }

    /// Attempts to insert `value` into the arena using existing capacity.
    ///
    /// This method will never allocate new capacity in the arena.
    ///
    /// If insertion succeeds, then the `value`'s index is returned. If
    /// insertion fails, then `Err(value)` is returned to give ownership of
    /// `value` back to the caller.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    ///
    /// match arena.try_insert(42) {
    ///     Ok(idx) => {
    ///         // Insertion succeeded.
    ///         assert_eq!(arena[idx], 42);
    ///     }
    ///     Err(x) => {
    ///         // Insertion failed.
    ///         assert_eq!(x, 42);
    ///     }
    /// };
    /// ```
    #[inline]
    pub fn try_insert(&mut self, value: T) -> Result<Index, T> {
        match self.try_alloc_next_index() {
            None => Err(value),
            Some(index) => {
                self.items[index.index] = Entry::Occupied {
                    generation: self.generation,
                    value,
                };
                Ok(index)
            }
        }
    }

    /// Attempts to insert the value returned by `create` into the arena using existing capacity.
    /// `create` is called with the new value's associated index, allowing values that know their own index.
    ///
    /// This method will never allocate new capacity in the arena.
    ///
    /// If insertion succeeds, then the new index is returned. If
    /// insertion fails, then `Err(create)` is returned to give ownership of
    /// `create` back to the caller.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::{Arena, Index};
    ///
    /// let mut arena = Arena::new();
    ///
    /// match arena.try_insert_with(|idx| (42, idx)) {
    ///     Ok(idx) => {
    ///         // Insertion succeeded.
    ///         assert_eq!(arena[idx].0, 42);
    ///         assert_eq!(arena[idx].1, idx);
    ///     }
    ///     Err(x) => {
    ///         // Insertion failed.
    ///     }
    /// };
    /// ```
    #[inline]
    pub fn try_insert_with<F: FnOnce(Index) -> T>(&mut self, create: F) -> Result<Index, F> {
        match self.try_alloc_next_index() {
            None => Err(create),
            Some(index) => {
                self.items[index.index] = Entry::Occupied {
                    generation: self.generation,
                    value: create(index),
                };
                Ok(index)
            }
        }
    }

    #[inline]
    fn try_alloc_next_index(&mut self) -> Option<Index> {
        match self.free_list_head {
            None => None,
            Some(i) => match self.items[i] {
                Entry::Occupied { .. } => panic!("corrupt free list"),
                Entry::Free { next_free } => {
                    self.free_list_head = next_free;
                    self.len += 1;
                    Some(Index {
                        index: i,
                        generation: self.generation,
                    })
                }
            },
        }
    }

    /// Insert `value` into the arena, allocating more capacity if necessary.
    ///
    /// The `value`'s associated index in the arena is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    ///
    /// let idx = arena.insert(42);
    /// assert_eq!(arena[idx], 42);
    /// ```
    #[inline]
    pub fn insert(&mut self, value: T) -> Index {
        match self.try_insert(value) {
            Ok(i) => i,
            Err(value) => self.insert_slow_path(value),
        }
    }

    /// Insert the value returned by `create` into the arena, allocating more capacity if necessary.
    /// `create` is called with the new value's associated index, allowing values that know their own index.
    ///
    /// The new value's associated index in the arena is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::{Arena, Index};
    ///
    /// let mut arena = Arena::new();
    ///
    /// let idx = arena.insert_with(|idx| (42, idx));
    /// assert_eq!(arena[idx].0, 42);
    /// assert_eq!(arena[idx].1, idx);
    /// ```
    #[inline]
    pub fn insert_with(&mut self, create: impl FnOnce(Index) -> T) -> Index {
        match self.try_insert_with(create) {
            Ok(i) => i,
            Err(create) => self.insert_with_slow_path(create),
        }
    }

    #[inline(never)]
    fn insert_slow_path(&mut self, value: T) -> Index {
        let len = self.items.len();
        self.reserve(len);
        self.try_insert(value)
            .map_err(|_| ())
            .expect("inserting will always succeed after reserving additional space")
    }

    #[inline(never)]
    fn insert_with_slow_path(&mut self, create: impl FnOnce(Index) -> T) -> Index {
        let len = self.items.len();
        self.reserve(len);
        self.try_insert_with(create)
            .map_err(|_| ())
            .expect("inserting will always succeed after reserving additional space")
    }

    /// Remove the element at index `i` from the arena.
    ///
    /// If the element at index `i` is still in the arena, then it is
    /// returned. If it is not in the arena, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let idx = arena.insert(42);
    ///
    /// assert_eq!(arena.remove(idx), Some(42));
    /// assert_eq!(arena.remove(idx), None);
    /// ```
    pub fn remove(&mut self, i: Index) -> Option<T> {
        if i.index >= self.items.len() {
            return None;
        }

        match self.items[i.index] {
            Entry::Occupied { generation, .. } if i.generation == generation => {
                let entry = mem::replace(
                    &mut self.items[i.index],
                    Entry::Free {
                        next_free: self.free_list_head,
                    },
                );
                self.generation += 1;
                self.free_list_head = Some(i.index);
                self.len -= 1;

                match entry {
                    Entry::Occupied {
                        generation: _,
                        value,
                    } => Some(value),
                    _ => unreachable!(),
                }
            }
            _ => None,
        }
    }

    /// Retains only the elements specified by the predicate.
    ///
    /// In other words, remove all indices such that `predicate(index, &value)` returns `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut crew = Arena::new();
    /// crew.extend(&["Jim Hawkins", "John Silver", "Alexander Smollett", "Israel Hands"]);
    /// let pirates = ["John Silver", "Israel Hands"]; // too dangerous to keep them around
    /// crew.retain(|_index, member| !pirates.contains(member));
    /// let mut crew_members = crew.iter().map(|(_, member)| **member);
    /// assert_eq!(crew_members.next(), Some("Jim Hawkins"));
    /// assert_eq!(crew_members.next(), Some("Alexander Smollett"));
    /// assert!(crew_members.next().is_none());
    /// ```
    pub fn retain(&mut self, mut predicate: impl FnMut(Index, &mut T) -> bool) {
        for i in 0..self.capacity() {
            let remove = match &mut self.items[i] {
                Entry::Occupied { generation, value } => {
                    let index = Index {
                        index: i,
                        generation: *generation,
                    };
                    if predicate(index, value) {
                        None
                    } else {
                        Some(index)
                    }
                }

                _ => None,
            };
            if let Some(index) = remove {
                self.remove(index);
            }
        }
    }

    /// Is the element at index `i` in the arena?
    ///
    /// Returns `true` if the element at `i` is in the arena, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let idx = arena.insert(42);
    ///
    /// assert!(arena.contains(idx));
    /// arena.remove(idx);
    /// assert!(!arena.contains(idx));
    /// ```
    pub fn contains(&self, i: Index) -> bool {
        self.get(i).is_some()
    }

    /// Get a shared reference to the element at index `i` if it is in the
    /// arena.
    ///
    /// If the element at index `i` is not in the arena, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let idx = arena.insert(42);
    ///
    /// assert_eq!(arena.get(idx), Some(&42));
    /// arena.remove(idx);
    /// assert!(arena.get(idx).is_none());
    /// ```
    pub fn get(&self, i: Index) -> Option<&T> {
        match self.items.get(i.index) {
            Some(Entry::Occupied { generation, value }) if *generation == i.generation => {
                Some(value)
            }
            _ => None,
        }
    }

    /// Get an exclusive reference to the element at index `i` if it is in the
    /// arena.
    ///
    /// If the element at index `i` is not in the arena, then `None` is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let idx = arena.insert(42);
    ///
    /// *arena.get_mut(idx).unwrap() += 1;
    /// assert_eq!(arena.remove(idx), Some(43));
    /// assert!(arena.get_mut(idx).is_none());
    /// ```
    pub fn get_mut(&mut self, i: Index) -> Option<&mut T> {
        match self.items.get_mut(i.index) {
            Some(Entry::Occupied { generation, value }) if *generation == i.generation => {
                Some(value)
            }
            _ => None,
        }
    }

    /// Get a pair of exclusive references to the elements at index `i1` and `i2` if it is in the
    /// arena.
    ///
    /// If the element at index `i1` or `i2` is not in the arena, then `None` is returned for this
    /// element.
    ///
    /// # Panics
    ///
    /// Panics if `i1` and `i2` are pointing to the same item of the arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let idx1 = arena.insert(0);
    /// let idx2 = arena.insert(1);
    ///
    /// {
    ///     let (item1, item2) = arena.get2_mut(idx1, idx2);
    ///
    ///     *item1.unwrap() = 3;
    ///     *item2.unwrap() = 4;
    /// }
    ///
    /// assert_eq!(arena[idx1], 3);
    /// assert_eq!(arena[idx2], 4);
    /// ```
    pub fn get2_mut(&mut self, i1: Index, i2: Index) -> (Option<&mut T>, Option<&mut T>) {
        let len = self.items.len();

        if i1.index == i2.index {
            assert!(i1.generation != i2.generation);

            if i1.generation > i2.generation {
                return (self.get_mut(i1), None);
            }
            return (None, self.get_mut(i2));
        }

        if i1.index >= len {
            return (None, self.get_mut(i2));
        } else if i2.index >= len {
            return (self.get_mut(i1), None);
        }

        let (raw_item1, raw_item2) = {
            let (xs, ys) = self.items.split_at_mut(cmp::max(i1.index, i2.index));
            if i1.index < i2.index {
                (&mut xs[i1.index], &mut ys[0])
            } else {
                (&mut ys[0], &mut xs[i2.index])
            }
        };

        let item1 = match raw_item1 {
            Entry::Occupied { generation, value } if *generation == i1.generation => Some(value),
            _ => None,
        };

        let item2 = match raw_item2 {
            Entry::Occupied { generation, value } if *generation == i2.generation => Some(value),
            _ => None,
        };

        (item1, item2)
    }

    /// Get the length of this arena.
    ///
    /// The length is the number of elements the arena holds.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// assert_eq!(arena.len(), 0);
    ///
    /// let idx = arena.insert(42);
    /// assert_eq!(arena.len(), 1);
    ///
    /// let _ = arena.insert(0);
    /// assert_eq!(arena.len(), 2);
    ///
    /// assert_eq!(arena.remove(idx), Some(42));
    /// assert_eq!(arena.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns true if the arena contains no elements
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// assert!(arena.is_empty());
    ///
    /// let idx = arena.insert(42);
    /// assert!(!arena.is_empty());
    ///
    /// assert_eq!(arena.remove(idx), Some(42));
    /// assert!(arena.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the capacity of this arena.
    ///
    /// The capacity is the maximum number of elements the arena can hold
    /// without further allocation, including however many it currently
    /// contains.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::with_capacity(10);
    /// assert_eq!(arena.capacity(), 10);
    ///
    /// // `try_insert` does not allocate new capacity.
    /// for i in 0..10 {
    ///     assert!(arena.try_insert(1).is_ok());
    ///     assert_eq!(arena.capacity(), 10);
    /// }
    ///
    /// // But `insert` will if the arena is already at capacity.
    /// arena.insert(0);
    /// assert!(arena.capacity() > 10);
    /// ```
    pub fn capacity(&self) -> usize {
        self.items.len()
    }

    /// Allocate space for `additional_capacity` more elements in the arena.
    ///
    /// # Panics
    ///
    /// Panics if this causes the capacity to overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::with_capacity(10);
    /// arena.reserve(5);
    /// assert_eq!(arena.capacity(), 15);
    /// # let _: Arena<usize> = arena;
    /// ```
    pub fn reserve(&mut self, additional_capacity: usize) {
        let start = self.items.len();
        let end = self.items.len() + additional_capacity;
        let old_head = self.free_list_head;
        self.items.reserve_exact(additional_capacity);
        self.items.extend((start..end).map(|i| {
            if i == end - 1 {
                Entry::Free {
                    next_free: old_head,
                }
            } else {
                Entry::Free {
                    next_free: Some(i + 1),
                }
            }
        }));
        self.free_list_head = Some(start);
    }

    /// Reduces allocated space to the minimum possible to fit all the elements in the arena.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// arena.extend([1, 2, 3, 4, 5].iter().copied());
    /// assert!(arena.capacity() >= 5);
    /// arena.shrink_to_fit();
    /// assert_eq!(arena.capacity(), 5);
    /// ```
    pub fn shrink_to_fit(&mut self) {
        self.items.truncate(
            self.items
                .iter()
                .rposition(|entry| matches!(entry, Entry::Occupied { .. }))
                .map(|n| n + 1)
                .unwrap_or(0),
        );
        self.items.shrink_to_fit();

        self.free_list_head = None;
        for (i, item) in self.items.iter_mut().enumerate() {
            match item {
                Entry::Occupied { .. } => (),
                Entry::Free { next_free } => {
                    *next_free = self.free_list_head;
                    self.free_list_head = Some(i);
                }
            }
        }
    }

    /// Iterate over shared references to the elements in this arena.
    ///
    /// Yields pairs of `(Index, &T)` items.
    ///
    /// Order of iteration is not defined.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// for i in 0..10 {
    ///     arena.insert(i * i);
    /// }
    ///
    /// for (idx, value) in arena.iter() {
    ///     println!("{} is at index {:?}", value, idx);
    /// }
    /// ```
    pub fn iter(&self) -> Iter<T> {
        Iter {
            len: self.len,
            inner: self.items.iter().enumerate(),
        }
    }

    /// Iterate over exclusive references to the elements in this arena.
    ///
    /// Yields pairs of `(Index, &mut T)` items.
    ///
    /// Order of iteration is not defined.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// for i in 0..10 {
    ///     arena.insert(i * i);
    /// }
    ///
    /// for (_idx, value) in arena.iter_mut() {
    ///     *value += 5;
    /// }
    /// ```
    pub fn iter_mut(&mut self) -> IterMut<T> {
        IterMut {
            len: self.len,
            inner: self.items.iter_mut().enumerate(),
        }
    }

    /// Iterate over elements of the arena and remove them.
    ///
    /// Yields pairs of `(Index, T)` items.
    ///
    /// Order of iteration is not defined.
    ///
    /// Note: All elements are removed even if the iterator is only partially consumed or not consumed at all.
    ///
    /// # Examples
    ///
    /// ```
    /// use generational_arena::Arena;
    ///
    /// let mut arena = Arena::new();
    /// let idx_1 = arena.insert("hello");
    /// let idx_2 = arena.insert("world");
    ///
    /// assert!(arena.get(idx_1).is_some());
    /// assert!(arena.get(idx_2).is_some());
    /// for (idx, value) in arena.drain() {
    ///     assert!((idx == idx_1 && value == "hello") || (idx == idx_2 && value == "world"));
    /// }
    /// assert!(arena.get(idx_1).is_none());
    /// assert!(arena.get(idx_2).is_none());
    /// ```
    pub fn drain(&mut self) -> Drain<T> {
        Drain {
            len: self.len,
            inner: self.items.drain(..).enumerate(),
        }
    }

    /// Given an i of `usize` without a generation, get a shared reference
    /// to the element and the matching `Index` of the entry behind `i`.
    ///
    /// This method is useful when you know there might be an element at the
    /// position i, but don't know its generation or precise Index.
    ///
    /// Use cases include using indexing such as Hierarchical BitMap Indexing or
    /// other kinds of bit-efficient indexing.
    ///
    /// You should use the `get` method instead most of the time.
    pub fn get_unknown_gen(&self, i: usize) -> Option<(&T, Index)> {
        match self.items.get(i) {
            Some(Entry::Occupied { generation, value }) => Some((
                value,
                Index {
                    generation: *generation,
                    index: i,
                },
            )),
            _ => None,
        }
    }

    /// Given an i of `usize` without a generation, get an exclusive reference
    /// to the element and the matching `Index` of the entry behind `i`.
    ///
    /// This method is useful when you know there might be an element at the
    /// position i, but don't know its generation or precise Index.
    ///
    /// Use cases include using indexing such as Hierarchical BitMap Indexing or
    /// other kinds of bit-efficient indexing.
    ///
    /// You should use the `get_mut` method instead most of the time.
    pub fn get_unknown_gen_mut(&mut self, i: usize) -> Option<(&mut T, Index)> {
        match self.items.get_mut(i) {
            Some(Entry::Occupied { generation, value }) => Some((
                value,
                Index {
                    generation: *generation,
                    index: i,
                },
            )),
            _ => None,
        }
    }
}

impl<T> IntoIterator for Arena<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            len: self.len,
            inner: self.items.into_iter(),
        }
    }
}

/// An iterator over the elements in an arena.
///
/// Yields `T` items.
///
/// Order of iteration is not defined.
///
/// # Examples
///
/// ```
/// use generational_arena::Arena;
///
/// let mut arena = Arena::new();
/// for i in 0..10 {
///     arena.insert(i * i);
/// }
///
/// for value in arena {
///     assert!(value < 100);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct IntoIter<T> {
    len: usize,
    inner: vec::IntoIter<Entry<T>>,
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(Entry::Free { .. }) => continue,
                Some(Entry::Occupied { value, .. }) => {
                    self.len -= 1;
                    return Some(value);
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some(Entry::Free { .. }) => continue,
                Some(Entry::Occupied { value, .. }) => {
                    self.len -= 1;
                    return Some(value);
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<T> FusedIterator for IntoIter<T> {}

impl<'a, T> IntoIterator for &'a Arena<T> {
    type Item = (Index, &'a T);
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

/// An iterator over shared references to the elements in an arena.
///
/// Yields pairs of `(Index, &T)` items.
///
/// Order of iteration is not defined.
///
/// # Examples
///
/// ```
/// use generational_arena::Arena;
///
/// let mut arena = Arena::new();
/// for i in 0..10 {
///     arena.insert(i * i);
/// }
///
/// for (idx, value) in &arena {
///     println!("{} is at index {:?}", value, idx);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct Iter<'a, T: 'a> {
    len: usize,
    inner: iter::Enumerate<slice::Iter<'a, Entry<T>>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = (Index, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((_, &Entry::Free { .. })) => continue,
                Some((
                    index,
                    &Entry::Occupied {
                        generation,
                        ref value,
                    },
                )) => {
                    self.len -= 1;
                    let idx = Index { index, generation };
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some((_, &Entry::Free { .. })) => continue,
                Some((
                    index,
                    &Entry::Occupied {
                        generation,
                        ref value,
                    },
                )) => {
                    self.len -= 1;
                    let idx = Index { index, generation };
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> FusedIterator for Iter<'a, T> {}

impl<'a, T> IntoIterator for &'a mut Arena<T> {
    type Item = (Index, &'a mut T);
    type IntoIter = IterMut<'a, T>;
    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// An iterator over exclusive references to elements in this arena.
///
/// Yields pairs of `(Index, &mut T)` items.
///
/// Order of iteration is not defined.
///
/// # Examples
///
/// ```
/// use generational_arena::Arena;
///
/// let mut arena = Arena::new();
/// for i in 0..10 {
///     arena.insert(i * i);
/// }
///
/// for (_idx, value) in &mut arena {
///     *value += 5;
/// }
/// ```
#[derive(Debug)]
pub struct IterMut<'a, T: 'a> {
    len: usize,
    inner: iter::Enumerate<slice::IterMut<'a, Entry<T>>>,
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = (Index, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((_, &mut Entry::Free { .. })) => continue,
                Some((
                    index,
                    &mut Entry::Occupied {
                        generation,
                        ref mut value,
                    },
                )) => {
                    self.len -= 1;
                    let idx = Index { index, generation };
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some((_, &mut Entry::Free { .. })) => continue,
                Some((
                    index,
                    &mut Entry::Occupied {
                        generation,
                        ref mut value,
                    },
                )) => {
                    self.len -= 1;
                    let idx = Index { index, generation };
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> FusedIterator for IterMut<'a, T> {}

/// An iterator that removes elements from the arena.
///
/// Yields pairs of `(Index, T)` items.
///
/// Order of iteration is not defined.
///
/// Note: All elements are removed even if the iterator is only partially consumed or not consumed at all.
///
/// # Examples
///
/// ```
/// use generational_arena::Arena;
///
/// let mut arena = Arena::new();
/// let idx_1 = arena.insert("hello");
/// let idx_2 = arena.insert("world");
///
/// assert!(arena.get(idx_1).is_some());
/// assert!(arena.get(idx_2).is_some());
/// for (idx, value) in arena.drain() {
///     assert!((idx == idx_1 && value == "hello") || (idx == idx_2 && value == "world"));
/// }
/// assert!(arena.get(idx_1).is_none());
/// assert!(arena.get(idx_2).is_none());
/// ```
#[derive(Debug)]
pub struct Drain<'a, T: 'a> {
    len: usize,
    inner: iter::Enumerate<vec::Drain<'a, Entry<T>>>,
}

impl<'a, T> Iterator for Drain<'a, T> {
    type Item = (Index, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some((_, Entry::Free { .. })) => continue,
                Some((index, Entry::Occupied { generation, value })) => {
                    let idx = Index { index, generation };
                    self.len -= 1;
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Drain<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next_back() {
                Some((_, Entry::Free { .. })) => continue,
                Some((index, Entry::Occupied { generation, value })) => {
                    let idx = Index { index, generation };
                    self.len -= 1;
                    return Some((idx, value));
                }
                None => {
                    debug_assert_eq!(self.len, 0);
                    return None;
                }
            }
        }
    }
}

impl<'a, T> ExactSizeIterator for Drain<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

impl<'a, T> FusedIterator for Drain<'a, T> {}

impl<T> Extend<T> for Arena<T> {
    fn extend<I: IntoIterator<Item = T>>(&mut self, iter: I) {
        for t in iter {
            self.insert(t);
        }
    }
}

impl<T> FromIterator<T> for Arena<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let iter = iter.into_iter();
        let (lower, upper) = iter.size_hint();
        let cap = upper.unwrap_or(lower);
        let cap = cmp::max(cap, 1);
        let mut arena = Arena::with_capacity(cap);
        arena.extend(iter);
        arena
    }
}

impl<T> ops::Index<Index> for Arena<T> {
    type Output = T;

    fn index(&self, index: Index) -> &Self::Output {
        self.get(index).expect("No element at index")
    }
}

impl<T> ops::IndexMut<Index> for Arena<T> {
    fn index_mut(&mut self, index: Index) -> &mut Self::Output {
        self.get_mut(index).expect("No element at index")
    }
}
