pub use super::lexer::Primitive;
use std::{ops::Range, path::PathBuf};

mod pointer;
pub use pointer::*;

mod ty;
pub use ty::*;

#[derive(Clone, Debug)]
pub struct Location {
    pub file_id: usize,
    pub span: Range<usize>,
}

impl Location {
    pub fn new(file_id: usize, span: Range<usize>) -> Self {
        Self { file_id, span }
    }
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub location: Location,
    pub identifier: String,
}

#[derive(Clone, Debug)]
pub struct Item {
    pub docs: Vec<String>,
    pub item: ItemType,
}

impl Item {
    pub fn location(&self) -> &Location {
        match &self.item {
            ItemType::Import(i) => &i.location,
            ItemType::Struct(s) => &s.location,
            ItemType::Function(f) => &f.location,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Function {
    pub location: Location,
    pub name: Identifier,
    pub args: Vec<(Identifier, Type)>,
    pub returns: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct Import {
    pub location: Location,
    pub path: String,
    pub resolved: Option<File>,
}

#[derive(Clone, Debug)]
pub enum ItemType {
    Import(Import),
    Struct(Struct),
    Function(Function),
}

#[derive(Clone, Debug)]
pub struct File {
    pub path: PathBuf,
    pub items: Vec<Item>,
}
