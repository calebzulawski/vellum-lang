use super::{Identifier, Type};
use codespan::Span;

pub enum Managed {
    With(Identifier),
    Embedded,
}

pub enum Constness {
    Const,
    Mut,
}

pub struct Pointer {
    pub span: Span,
    pub managed: Option<Managed>,
    pub constness: Constness,
    pub ty: Box<Type>,
}
