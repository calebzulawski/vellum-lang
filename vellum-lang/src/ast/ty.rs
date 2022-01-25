use super::{Identifier, Pointer};
use codespan::Span;

pub use crate::lexer::Primitive;

pub struct Struct {
    pub span: Span,
    pub name: Identifier,
    pub fields: Vec<(Identifier, Type)>,
}

pub enum Type {
    Primitive { span: Span, primitive: Primitive },
    Pointer(Pointer),
    Identifier(Identifier),
}
