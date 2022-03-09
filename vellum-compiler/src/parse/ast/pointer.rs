use super::{Location, Type};

#[derive(Clone, Debug)]
pub enum PointerModifier {
    Const,
    Mut,
}

#[derive(Clone, Debug)]
pub struct Pointer {
    pub location: Location,
    pub modifier: PointerModifier,
    pub ty: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct StringPointer {
    pub location: Location,
    pub modifier: PointerModifier,
}

#[derive(Clone, Debug)]
pub struct Owned {
    pub location: Location,
    pub ty: Box<Type>,
}
