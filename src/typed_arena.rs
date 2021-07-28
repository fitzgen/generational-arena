use crate::{
    prelude::*,
    Index,
};

///
#[derive(Debug, Clone)]
pub struct TypedArena<T> {
    inner: Arena<T>,
}

impl<T> Default for TypedArena<T> {
    #[inline(always)]
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

    ///
    #[inline(always)]
    pub fn remove(&mut self, i: TypedIndex<T>) -> Option<T> {
        self.inner.typed_remove(i)
    }

    ///
    #[inline(always)]
    pub fn retain(&mut self, mut predicate: impl FnMut(TypedIndex<T>, &mut T) -> bool) {
        self.inner.retain(|i, e| predicate(i.into(), e))
    }

    ///
    #[inline(always)]
    pub fn contains(&self, i: TypedIndex<T>) -> bool {
        // self.inner.contains(Index::from_raw_parts(a, b))
        todo!()
    }

    ///
    #[inline(always)]
    pub fn get(&self, i: TypedIndex<T>) -> Option<&T> {
        self.inner.typed_get(i)
    }

    ///
    #[inline(always)]
    pub fn get_mut(&mut self, i: TypedIndex<T>) -> Option<&mut T> {
        self.inner.typed_get_mut(i)
    }

    ///
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    ///
    #[inline(always)]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    ///
    #[inline(always)]
    pub fn reserve(&mut self, additional_capacity: usize) {
        self.inner.reserve(additional_capacity)
    }

    ///
    #[inline(always)]
    pub fn iter(&self) -> TypedIter<T> {
        self.inner.typed_iter()
    }

    ///
    #[inline(always)]
    pub fn iter_mut(&mut self) -> TypedIterMut<T> {
        self.inner.typed_iter_mut()
    }

    ///
    #[inline(always)]
    pub fn get_unknown_gen(&self, i: usize) -> Option<(TypedIndex<T>, &T)> {
        self.inner.typed_get_unknown_gen(i)
    }

    ///
    #[inline(always)]
    pub fn get_unknown_gen_mut(&mut self, i: usize) -> Option<(TypedIndex<T>, &mut T)> {
        self.inner.typed_get_unknown_gen_mut(i)
    }

    ///
    pub fn raw_load(max_index: usize, i: impl IntoIterator<Item = (TypedIndex<T>, T)>) -> Self {
        let i = i.into_iter();
        let size_hint = i.size_hint();
        todo!();
    }
}

impl<T> std::ops::Index<TypedIndex<T>> for TypedArena<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: TypedIndex<T>) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> std::ops::IndexMut<TypedIndex<T>> for TypedArena<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: TypedIndex<T>) -> &mut Self::Output {
        &mut self.inner[index]
    }
}

impl<T> std::ops::Index<&TypedIndex<T>> for TypedArena<T> {
    type Output = T;

    #[inline(always)]
    fn index(&self, index: &TypedIndex<T>) -> &Self::Output {
        &self.inner[index]
    }
}

impl<T> std::ops::IndexMut<&TypedIndex<T>> for TypedArena<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: &TypedIndex<T>) -> &mut Self::Output {
        &mut self.inner[index]
    }
}
