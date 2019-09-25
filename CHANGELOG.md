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
