use codespan::Span;

mod pointer;
pub use pointer::*;

mod ty;
pub use ty::*;

pub struct Identifier {
    pub span: Span,
    pub identifier: String,
}

pub enum Item {
    Struct(Struct),
}
