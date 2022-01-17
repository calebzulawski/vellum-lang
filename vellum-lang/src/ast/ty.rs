use super::{Identifier, Pointer};
use codespan::Span;

pub enum Primitive {
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
}

pub enum Type {
    Primitive { span: Span, ty: Primitive },
    Pointer(Pointer),
    Identifier(Identifier),
}
