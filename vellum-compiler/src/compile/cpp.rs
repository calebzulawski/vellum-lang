use super::{Compile, Items};
use crate::parse::{Context, ast};
use askama::Template;
use codespan_reporting::diagnostic::Diagnostic;
use std::{
    fs::OpenOptions,
    io::{Error, Write},
    path::Path,
};

#[derive(Template)]
#[template(path = "c++/compile.hpp")]
struct CppTemplate {
    items: Items,
}

pub(super) fn compile(context: &mut Context, options: Compile, items: Items) -> Result<(), ()> {
    let file_name = Path::new(&options.file)
        .with_extension("hpp")
        .file_name()
        .unwrap()
        .to_owned();
    let output_file = if let Some(output_dir) = options.resolve_output_dir() {
        output_dir.join(file_name)
    } else {
        file_name.into()
    };
    if let Err(e) = compile_impl(items, &output_file) {
        context.report(
            &Diagnostic::error()
                .with_message(format!("error writing to `{}`", output_file.display()))
                .with_notes(vec![format!("{}", e)]),
        );
        Err(())
    } else {
        Ok(())
    }
}

fn compile_impl(items: Items, output_file: &Path) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_file)?;

    let template = CppTemplate { items };
    write!(file, "{}", template.render().unwrap())?;
    Ok(())
}

struct DisplayType<'a>(&'a ast::Type);

impl std::fmt::Display for DisplayType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.0 {
            ast::Type::Primitive { primitive, .. } => {
                let s = match primitive {
                    ast::Primitive::Bool => "bool",
                    ast::Primitive::I8 => "int8_t",
                    ast::Primitive::I16 => "int16_t",
                    ast::Primitive::I32 => "int32_t",
                    ast::Primitive::I64 => "int64_t",
                    ast::Primitive::Isize => "ssize_t",
                    ast::Primitive::U8 => "uint8_t",
                    ast::Primitive::U16 => "uint16_t",
                    ast::Primitive::U32 => "uint32_t",
                    ast::Primitive::U64 => "uint64_t",
                    ast::Primitive::Usize => "size_t",
                };
                write!(f, "{}", s)?;
            }
            ast::Type::Pointer(p) => {
                let modifier = match p.modifier {
                    ast::PointerModifier::Const => " const",
                    ast::PointerModifier::Mut => "",
                };
                write!(f, "{}{} *", DisplayType(p.ty.as_ref()), modifier)?;
            }
            ast::Type::String(s) => {
                let modifier = match s.modifier {
                    ast::PointerModifier::Const => " const",
                    ast::PointerModifier::Mut => "",
                };
                write!(f, "char{} *", modifier)?;
            }
            ast::Type::Slice(s) => {
                let modifier = match s.modifier {
                    ast::PointerModifier::Const => "const ",
                    ast::PointerModifier::Mut => "",
                };
                write!(
                    f,
                    "vellum::slice<{}{}>",
                    modifier,
                    DisplayType(s.ty.as_ref())
                )?;
            }
            ast::Type::Owned(p) => {
                write!(f, "vellum::owned<{}>", DisplayType(&p.ty))?;
            }
            ast::Type::FunctionPointer(ast::FunctionPointer {
                fn_ty,
                args,
                returns,
                ..
            }) => {
                let fn_ty_name = match fn_ty {
                    ast::FunctionType::Function => "function",
                    ast::FunctionType::Closure => "closure",
                };
                write!(
                    f,
                    "vellum::{}<{} (",
                    fn_ty_name,
                    DisplayType(returns.as_ref())
                )?;
                if !args.is_empty() {
                    for arg in args.iter().take(args.len() - 1) {
                        write!(f, "{}, ", DisplayType(&arg.1))?;
                    }
                    write!(f, "{}", DisplayType(&args.last().unwrap().1))?;
                }
                write!(f, ")>")?;
            }
            ast::Type::Array(a) => {
                write!(f, "std::array<{}, {}>", DisplayType(&a.ty), a.len)?;
            }
            ast::Type::Identifier(i) => write!(f, "{}", i.identifier)?,
        }
        Ok(())
    }
}

mod filters {
    use super::*;

    pub fn ty(ty: &ast::Type, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(DisplayType(ty).to_string())
    }
}
