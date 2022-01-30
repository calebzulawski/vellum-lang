use super::{Identifier, Pointer};
use codespan::Span;

pub use super::Primitive;

pub struct Field {
    pub docs: Vec<String>,
    pub name: Identifier,
    pub ty: Type,
}

pub struct Struct {
    pub span: Span,
    pub name: Identifier,
    pub fields: Vec<Field>,
}

pub enum Type {
    Primitive { span: Span, primitive: Primitive },
    Pointer(Pointer),
    Identifier(Identifier),
}
