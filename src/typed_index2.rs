use crate::TypedIndex;

///
pub struct TypedIndex2<A, B> {
    fst: TypedIndex<A>,
    snd: TypedIndex<B>,
}

impl<A, B> TypedIndex2<A, B> {
    ///
    pub fn new(fst: TypedIndex<A>, snd: TypedIndex<B>) -> Self {
        Self { fst, snd }
    }

    ///
    pub fn fst(&self) -> TypedIndex<A> {
        self.fst
    }

    ///
    pub fn snd(&self) -> TypedIndex<B> {
        self.snd
    }
}

impl<A, B> Clone for TypedIndex2<A, B> {
    fn clone(&self) -> Self {
        Self::new(self.fst, self.snd)
    }
}

impl<A, B> Copy for TypedIndex2<A, B> {}

impl<A, B> PartialEq for TypedIndex2<A, B> {
    fn eq(&self, other: &Self) -> bool {
        self.fst == other.fst && self.snd == other.snd
    }
}

impl<A, B> Eq for TypedIndex2<A, B> {}

impl<A, B> std::ops::Add<TypedIndex<B>> for TypedIndex<A> {
    type Output = TypedIndex2<A, B>;
    fn add(self, other: TypedIndex<B>) -> Self::Output {
        Self::Output::new(self, other)
    }
}

impl<A, B> std::fmt::Debug for TypedIndex2<A, B> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "TypedIndex2 {{ fst: {:?}, snd: {:?} }}",
            self.fst, self.snd
        )
    }
}
