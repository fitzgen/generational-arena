#![cfg(feature = "serde")]

extern crate generational_arena;
#[macro_use]
extern crate serde;
extern crate bincode;
extern crate serde_test;

use generational_arena::{Arena, Index};
use serde::{Deserialize, Serialize};
use serde_test::{assert_ser_tokens, Token};
use std::fmt::Debug;

#[test]
fn deserialized_arena_holds_same_values_with_original_arena() {
    let mut arena = Arena::new();
    let a = arena.insert("apple");
    let b0 = arena.insert("banana");
    let c = arena.insert("cherry");
    let d = arena.insert("durian");
    assert_eq!(arena.remove(b0), Some("banana"));
    let b1 = arena.insert("bacon");
    assert_eq!(arena.remove(d), Some("durian"));

    let bytes = bincode::serialize(&arena).expect("arena must be serialized");
    let de_arena = bincode::deserialize::<Arena<&str>>(&bytes).expect("arena must be deserialized");

    // Check both arenas behave the same way
    for arena in &mut [arena, de_arena] {
        assert_eq!(arena.get(a), Some(&"apple"));
        assert_eq!(arena.get(b0), None);
        assert_eq!(arena.get(b1), Some(&"bacon"));
        assert_eq!(arena.get(c), Some(&"cherry"));
        assert_eq!(arena.get(d), None);
    }
}

#[test]
fn deserialized_index_can_be_used_in_the_same_way_as_original_index() {
    let mut arena = Arena::new();
    let a = arena.insert("apple");
    let b0 = arena.insert("banana");
    let c = arena.insert("cherry");
    let d = arena.insert("durian");
    assert_eq!(arena.remove(b0), Some("banana"));
    let b1 = arena.insert("bacon");
    assert_eq!(arena.remove(d), Some("durian"));

    for idx in &[a, b0, b1, c, d] {
        let bytes = bincode::serialize(&idx).expect("index must be serialized");
        let de_idx = bincode::deserialize::<Index>(&bytes).expect("index must be deserialized");
        assert_eq!(arena.get(*idx), arena.get(de_idx));
    }
}

#[test]
fn sparse_deserialized_arena_can_use_whole_elements_in_free_list() {
    let capacity = 100;
    let len = 3;
    // To create an Arena with arbitrary index/generation, create a Vec with
    // the same serialized representation as the Arena
    let mut seq: Vec<Option<(u64, &'static str)>> = vec![None; capacity];
    seq[8] = Some((10, "foo"));
    seq[45] = Some((80, "bar"));
    seq[99] = Some((123, "baz"));
    let mut tokens = vec![Token::Seq {
        len: Some(capacity),
    }];
    for i in 0..capacity {
        match i {
            8 => {
                tokens.extend(&[
                    Token::Some,
                    Token::Tuple { len: 2 },
                    Token::U64(10),
                    Token::BorrowedStr("foo"),
                    Token::TupleEnd,
                ]);
            }
            45 => {
                tokens.extend(&[
                    Token::Some,
                    Token::Tuple { len: 2 },
                    Token::U64(80),
                    Token::BorrowedStr("bar"),
                    Token::TupleEnd,
                ]);
            }
            99 => {
                tokens.extend(&[
                    Token::Some,
                    Token::Tuple { len: 2 },
                    Token::U64(123),
                    Token::BorrowedStr("baz"),
                    Token::TupleEnd,
                ]);
            }
            _ => tokens.push(Token::None),
        }
    }
    tokens.push(Token::SeqEnd);
    serde_test::assert_tokens(&seq, &tokens);
    let bytes = bincode::serialize(&seq).expect("vec must be serialized");

    let mut arena =
        bincode::deserialize::<Arena<&str>>(&bytes).expect("arena must be deserialized");
    assert_eq!(arena.capacity(), capacity);
    for _i in 0..(capacity - len) {
        arena.insert("quux");
    }
    assert_eq!(arena.capacity(), capacity);
}

#[test]
fn empty_arena_can_be_serialized_and_deserialized() {
    let arena = Arena::<()>::new();
    let cap = arena.capacity();
    let mut tokens = vec![Token::Seq { len: Some(cap) }];
    tokens.extend(vec![Token::None; cap]);
    tokens.push(Token::SeqEnd);
    assert_tokens(&arena, &tokens);
}

#[test]
fn fully_occupied_arena_can_be_serialized_and_deserialized() {
    let mut arena = Arena::with_capacity(30); // 30 is greater than default capacity(4)
    let mut tokens = vec![Token::Seq { len: Some(30) }];
    for i in 0..arena.capacity() {
        let _ = arena.insert(i * i);
        tokens.extend(&[
            Token::Some,
            Token::Tuple { len: 2 },
            Token::U64(0),
            Token::U64((i * i) as u64),
            Token::TupleEnd,
        ]);
    }
    tokens.push(Token::SeqEnd);
    assert_tokens(&arena, &tokens);
}

/// Arena wrapper struct for comparing two arenas
///
/// `serde_test::assert_tokens` requires the value implements `PartialEq`,
/// but `Arena` does not implement it.
#[derive(Debug, Serialize, Deserialize)]
struct ArenaCompare<T>(Arena<T>);

impl<'a, T> PartialEq for ArenaCompare<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.iter().eq(other.0.iter())
    }
}

fn assert_de_tokens<T>(value: &Arena<T>, tokens: &[Token])
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + Clone + Debug,
{
    let comp = ArenaCompare(value.clone());
    let mut comp_tokens = vec![Token::NewtypeStruct {
        name: "ArenaCompare",
    }];
    comp_tokens.extend(tokens);
    serde_test::assert_de_tokens(&comp, &comp_tokens);
}

fn assert_tokens<T>(value: &Arena<T>, tokens: &[Token])
where
    T: Serialize + for<'de> Deserialize<'de> + PartialEq + Clone + Debug,
{
    assert_ser_tokens(value, tokens);
    assert_de_tokens(value, tokens);
}
