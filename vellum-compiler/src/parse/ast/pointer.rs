use super::Type;
use codespan::Span;

pub enum PointerModifier {
    Const,
    Mut,
    Owned,
}

pub struct Pointer {
    pub span: Span,
    pub modifier: PointerModifier,
    pub ty: Box<Type>,
}
