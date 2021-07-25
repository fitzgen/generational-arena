// use crate::prelude::Index;
use crate::prelude::{
    Arena,
    TypedIndex,
};
///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DynIndex {
    inner: crate::Index,
    type_id: std::any::TypeId,
    name: &'static str,
}

unsafe impl Send for DynIndex {}
unsafe impl Sync for DynIndex {}

impl<T: 'static> From<TypedIndex<T>> for DynIndex {
    fn from(a: TypedIndex<T>) -> Self {
        let type_id = std::any::TypeId::of::<T>();
        let name = std::any::type_name::<T>();
        Self {
            inner: a.inner(),
            type_id,
            name,
        }
    }
}

impl<T: 'static> From<DynIndex> for TypedIndex<T> {
    fn from(idx: DynIndex) -> Self {
        let type_id = std::any::TypeId::of::<T>();
        assert!(idx.type_id == type_id);
        idx.inner.into()
    }
}

impl<T> std::ops::Index<DynIndex> for Arena<T> {
    type Output = T;
    #[inline(always)]
    fn index(&self, index: DynIndex) -> &Self::Output {
        // &self[index.inner]
        todo!()
    }
}

impl<T> std::ops::IndexMut<DynIndex> for Arena<T> {
    #[inline(always)]
    fn index_mut(&mut self, index: DynIndex) -> &mut Self::Output {
        // &mut self[index.inner]
        todo!()
    }
}
