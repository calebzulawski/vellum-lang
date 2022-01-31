use super::{Location, Type};

pub enum PointerModifier {
    Const,
    Mut,
    Owned,
}

pub struct Pointer {
    pub location: Location,
    pub modifier: PointerModifier,
    pub ty: Box<Type>,
}
