use super::{Identifier, Location, Pointer};

pub use super::Primitive;

#[derive(Clone, Debug)]
pub struct Field {
    pub docs: Vec<String>,
    pub name: Identifier,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub location: Location,
    pub name: Identifier,
    pub fields: Option<Vec<Field>>,
}

#[derive(Clone, Debug)]
pub enum Type {
    Primitive {
        location: Location,
        primitive: Primitive,
    },
    Pointer(Pointer),
    Identifier(Identifier),
}
