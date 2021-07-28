use crate::{
    Arena,
    Index,
};
use std::{
    fmt::Debug,
    hash::Hash,
};

///
pub struct TypedIndex<T> {
    inner: Index,
    ph: std::marker::PhantomData<T>,
}

impl<T> TypedIndex<T> {
    ///
    #[inline(always)]
    pub fn from_raw_parts(a: usize, b: u64) -> Self {
        Self::new(Index::from_raw_parts(a, b))
    }

    ///
    #[inline(always)]
    pub fn into_raw_parts(self) -> (usize, u64) {
        self.inner.into_raw_parts()
    }

    ///
    #[inline(always)]
    pub fn new(inner: Index) -> Self {
        Self {
            inner,
            ph: Default::default(),
        }
    }

    ///
    #[inline(always)]
    pub fn index(&self) -> usize {
        self.inner.index()
    }

    ///
    #[inline(always)]
    pub fn generation(&self) -> u64 {
        self.inner.generation
    }

    ///
    #[inline]
    pub fn inner(&self) -> Index {
        self.inner
    }
}

impl<T> Clone for TypedIndex<T> {
    #[inline]
    fn clone(&self) -> Self {
        Self::new(self.inner)
    }
}

impl<T> Copy for TypedIndex<T> {}

impl<T> PartialEq for TypedIndex<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl<T> Eq for TypedIndex<T> {}

impl<T> Hash for TypedIndex<T> {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl<T> From<Index> for TypedIndex<T> {
    #[inline]
    fn from(a: Index) -> Self {
        Self::new(a)
    }
}

impl<T> Debug for TypedIndex<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(std::any::type_name::<Self>())
            .field("inner: ", &self.inner)
            .finish()
    }
}

impl<T> PartialOrd for TypedIndex<T> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.inner.index().partial_cmp(&other.inner.index())
    }
}

impl<T> Ord for TypedIndex<T> {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

unsafe impl<T> Send for TypedIndex<T> {}
unsafe impl<T> Sync for TypedIndex<T> {}

impl<T> std::ops::Index<TypedIndex<T>> for Arena<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: TypedIndex<T>) -> &Self::Output {
        &self[index.inner]
    }
}

impl<T> std::ops::IndexMut<TypedIndex<T>> for Arena<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: TypedIndex<T>) -> &mut Self::Output {
        &mut self[index.inner]
    }
}

impl<T> std::ops::Index<&TypedIndex<T>> for Arena<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: &TypedIndex<T>) -> &Self::Output {
        &self[index.inner]
    }
}

impl<T> std::ops::IndexMut<&TypedIndex<T>> for Arena<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: &TypedIndex<T>) -> &mut Self::Output {
        &mut self[index.inner]
    }
}

impl<T> Arena<T> {
    ///
    #[inline(always)]
    pub fn typed_insert(&mut self, value: T) -> TypedIndex<T> {
        TypedIndex::new(self.insert(value))
    }

    ///
    #[inline(always)]
    pub fn typed_insert_with(&mut self, create: impl FnOnce(TypedIndex<T>) -> T) -> TypedIndex<T> {
        TypedIndex::new(self.insert_with(|index| {
            let idx = TypedIndex::new(index);
            create(idx)
        }))
    }

    ///
    #[inline(always)]
    pub fn typed_remove(&mut self, index: TypedIndex<T>) -> Option<T> {
        self.remove(index.inner)
    }
}
