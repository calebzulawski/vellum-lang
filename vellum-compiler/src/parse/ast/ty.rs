use super::{Identifier, Location, Pointer};

pub use super::Primitive;

pub struct Field {
    pub docs: Vec<String>,
    pub name: Identifier,
    pub ty: Type,
}

pub struct Struct {
    pub location: Location,
    pub name: Identifier,
    pub fields: Vec<Field>,
}

pub enum Type {
    Primitive {
        location: Location,
        primitive: Primitive,
    },
    Pointer(Pointer),
    Identifier(Identifier),
}
