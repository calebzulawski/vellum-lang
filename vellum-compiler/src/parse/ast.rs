pub use super::lexer::Primitive;
use core::ops::Range;

mod pointer;
pub use pointer::*;

mod ty;
pub use ty::*;

pub struct Location {
    pub file_id: usize,
    pub span: Range<usize>,
}

impl Location {
    pub fn new(file_id: usize, span: Range<usize>) -> Self {
        Self { file_id, span }
    }
}

pub struct Identifier {
    pub location: Location,
    pub identifier: String,
}

pub struct Item {
    pub docs: Vec<String>,
    pub item: ItemType,
}

pub struct Import {
    pub location: Location,
    pub path: String,
}

pub enum ItemType {
    Import(Import),
    Struct(Struct),
}
