use codespan::Span;

mod pointer;
pub use pointer::*;

mod ty;
pub use ty::*;

pub struct Identifier {
    pub span: Span,
    pub identifier: String,
}

pub struct Item {
    pub docs: Vec<String>,
    pub item: ItemType,
}

pub struct Import {
    pub span: Span,
    pub path: String,
}

pub enum ItemType {
    Import(Import),
    Struct(Struct),
}
