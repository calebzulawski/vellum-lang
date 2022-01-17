use super::{Identifier, Type};
use codespan::Span;

pub enum PointerScope {
    Managed(Identifier),
    Tracked,
}

pub enum PointerConstness {
    Const,
    Mut,
}

pub struct Pointer {
    pub span: Span,
    pub scope: Option<PointerScope>,
    pub constness: PointerConstness,
    pub ty: Box<Type>,
}
