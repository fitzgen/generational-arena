use crate::prelude::*;

///
#[derive(Debug)]
pub struct TypedArena<T> {
    inner: Arena<T>,
}
