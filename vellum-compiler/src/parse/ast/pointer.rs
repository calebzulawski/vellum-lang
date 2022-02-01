use super::{Location, Type};

#[derive(Clone, Debug)]
pub enum PointerModifier {
    Const,
    Mut,
    Owned,
}

#[derive(Clone, Debug)]
pub struct Pointer {
    pub location: Location,
    pub modifier: PointerModifier,
    pub ty: Box<Type>,
}
