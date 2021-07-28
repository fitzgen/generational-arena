use crate::prelude::*;

///
#[derive(Debug)]
pub struct TypedArena<T> {
    inner: Arena<T>,
}

impl<T> TypedArena<T> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
        }
    }
}

impl<T> TypedArena<T> {
    ///
    #[inline(always)]
    fn from(arena: Arena<T>) -> Self {
        Self { inner: arena }
    }

    ///
    #[inline(always)]
    pub fn new() -> Self {
        Self::from(Arena::new())
    }

    ///
    #[inline(always)]
    pub fn with_capacity(n: usize) -> Self {
        Self::from(Arena::with_capacity(n))
    }

    ///
    #[inline(always)]
    pub fn clear(&mut self) {
        self.inner.clear()
    }

    ///
    #[inline(always)]
    pub fn try_insert(&mut self, value: T) -> Result<TypedIndex<T>, T> {
        todo!()
        // match self.inner.try_insert(value) {

        // }
    }

    ///
    pub fn try_insert_with<F: FnOnce(TypedIndex<T>) -> T>(
        &mut self,
        create: F,
    ) -> Result<TypedIndex<T>, F> {
        todo!()
    }

    //
    // fn try_alloc_next_index(&mut self) -> Option<Index> {
    // }

    ///
    #[inline(always)]
    pub fn insert(&mut self, value: T) -> TypedIndex<T> {
        self.inner.typed_insert(value)
    }

    ///
    #[inline(always)]
    pub fn insert_with(&mut self, create: impl FnOnce(TypedIndex<T>) -> T) -> TypedIndex<T> {
        self.inner.typed_insert_with(create)
    }
}
