# 0.2.8

Released 2020-05-18.

* Add a `Default` implementation for `Arena`
* Add the `insert_with` and `try_insert_with` methods to `Arena`. These methods
  allow creating a value while inserting it, and the function to create the
  value is given the its id in the arena. For example, this allows a struct that
  is in an arena to have a member field that is its id within the arena.

# 0.2.7

Released 2020-01-03.

* Fixed a bug in `Arena::retain` where not every element was always considered
  for retention. See https://github.com/fitzgen/generational-arena/pull/28 for
  details.

# 0.2.6

Released 2019-11-11.

* Added `Arena::get_unknown_gen[_mut]` methods for the rare cases where you need
  to get the item and `Index` at a given offset within the arena.

# 0.2.5

Yanked because of bad `cargo publish`.

# 0.2.4

Released 2019-11-04.

* The `retain` method now gives mutable references to the arena's items, rather
  than shared references. This matches `Vec::retain`.
* Upgraded to 2018 edition.

# 0.2.3

Released 2019-09-25.

* Add methods for converting `Index` into and from its raw parts.

# 0.2.2

Released 2019-03-12.

* Add a `retain` method analogous to `Vec::retain`.

# 0.2.1

Released 2019-01-22.

* Bad indexing into an arena will now panic with a message explaining what
  happened instead of the generic unwrap panic message.

# 0.2.0

Released 2018-11-28.

* Added support for `serde` serialization and deserialization. Enable the
  "serde" feature to access it.
* Added `clear` method to empty an arena.
* Added a `drain` method to iterate over items and remove them from the arena at
  the same time.
* Implemented `ExactSizeIterator`, `DoubleEndedIterator`, and `FusedIterator`
  for our various iterators. This also gave a nice speed up to iteration.
* Added the `get2_mut` method to get two distinct items out of the arena mutably
  at the same time.

# 0.1.0
