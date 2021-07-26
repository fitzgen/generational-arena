// use crate::prelude::Index;
use crate::prelude::{
    Arena,
    TypedIndex,
};
fn type_id<T: 'static>() -> std::any::TypeId {
    std::any::TypeId::of::<T>()
}

///
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DynIndex {
    inner: crate::Index,
    type_id: std::any::TypeId,
    name: &'static str,
}

impl PartialOrd for DynIndex {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.inner.partial_cmp(&other.inner)
    }
}

impl Ord for DynIndex {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl std::hash::Hash for DynIndex {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state)
    }
}

impl DynIndex {
    ///
    #[inline]
    pub fn matches<T: 'static>(&self) -> bool {
        self.type_id == type_id::<T>()
    }
}

unsafe impl Send for DynIndex {}
unsafe impl Sync for DynIndex {}

impl<T: 'static> From<TypedIndex<T>> for DynIndex {
    fn from(a: TypedIndex<T>) -> Self {
        let type_id = type_id::<T>();
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
        //todo: make these debug asserts?
        let type_id = std::any::TypeId::of::<T>();
        assert!(idx.type_id == type_id);
        idx.inner.into()
    }
}

// impl<T: 'static> std::ops::Index<DynIndex> for Arena<T> {
//     type Output = T;
//     #[inline(always)]
//     fn index(&self, index: DynIndex) -> &Self::Output {
//         //todo: make these debug asserts?
//         let type_id = type_id::<T>();
//         assert!(index.type_id == type_id);
//         &self[index.inner]
//     }
// }

// impl<T: 'static> std::ops::IndexMut<DynIndex> for Arena<T> {
//     #[inline(always)]
//     fn index_mut(&mut self, index: DynIndex) -> &mut Self::Output {
//         //todo: make these debug asserts?
//         let type_id = type_id::<T>();
//         assert!(index.type_id == type_id);
//         &mut self[index.inner]
//     }
// }
