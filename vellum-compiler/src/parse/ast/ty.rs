use super::{Identifier, Location, Owned, Pointer, Slice, StringPointer};

pub use super::Primitive;

#[derive(Clone, Debug)]
pub struct Field {
    pub docs: Vec<String>,
    pub name: Identifier,
    pub ty: Type,
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub location: Location,
    pub name: Identifier,
    pub fields: Option<Vec<Field>>,
}

#[derive(Copy, Clone, Debug)]
pub enum FunctionType {
    Function,
    Closure,
}

#[derive(Clone, Debug)]
pub struct FunctionPointer {
    pub location: Location,
    pub fn_ty: FunctionType,
    pub args: Vec<(Identifier, Type)>,
    pub returns: Box<Type>,
}

#[derive(Clone, Debug)]
pub struct Array {
    pub location: Location,
    pub ty: Box<Type>,
    pub len: u64,
}

#[derive(Clone, Debug)]
pub enum Type {
    Primitive {
        location: Location,
        primitive: Primitive,
    },
    Pointer(Pointer),
    String(StringPointer),
    Slice(Slice),
    Owned(Owned),
    FunctionPointer(FunctionPointer),
    Array(Array),
    Identifier(Identifier),
}

impl Type {
    pub fn iter_tree<'a>(&'a self) -> TypeIterator<'a> {
        return TypeIterator { stack: vec![self] };
    }
}

pub struct TypeIterator<'a> {
    stack: Vec<&'a Type>,
}

impl<'a> Iterator for TypeIterator<'a> {
    type Item = &'a Type;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(ty) = self.stack.pop() {
            match &ty {
                Type::Primitive {
                    location: _,
                    primitive: _,
                } => {}
                Type::Pointer(p) => self.stack.push(p.ty.as_ref()),
                Type::String(_) => {}
                Type::Slice(s) => self.stack.push(s.ty.as_ref()),
                Type::Owned(o) => self.stack.push(o.ty.as_ref()),
                Type::FunctionPointer(f) => {
                    for (_, ty) in f.args.iter() {
                        self.stack.push(&ty);
                    }
                    self.stack.push(f.returns.as_ref());
                }
                Type::Array(a) => self.stack.push(a.ty.as_ref()),
                Type::Identifier(_) => {}
            }
            Some(ty)
        } else {
            None
        }
    }
}
