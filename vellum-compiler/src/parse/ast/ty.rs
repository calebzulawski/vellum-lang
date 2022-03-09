use super::{Identifier, Location, Owned, Pointer, StringPointer};

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

#[derive(Copy, Clone, Debug)]
pub enum FunctionType {
    Function,
    Closure,
}

#[derive(Clone, Debug)]
pub struct FunctionPointer {
    pub location: Location,
    pub fn_ty: FunctionType,
    pub args: Vec<(Identifier, Type)>,
    pub returns: Box<Type>,
}

#[derive(Clone, Debug)]
pub enum Type {
    Primitive {
        location: Location,
        primitive: Primitive,
    },
    Pointer(Pointer),
    String(StringPointer),
    Owned(Owned),
    FunctionPointer(FunctionPointer),
    Identifier(Identifier),
}
