use super::{Compile, Items, Mode};
use crate::parse::{Context, ast};
use askama::Template;
use codespan_reporting::diagnostic::Diagnostic;
use std::{
    fs::OpenOptions,
    io::{Error, Write},
    path::{Path, PathBuf},
};

#[derive(Template)]
#[template(path = "c++/import.hpp")]
struct CppHeaderTemplate {
    items: Items,
}

#[derive(Template)]
#[template(path = "c++/export.inl", escape = "none")]
struct CppExportInlineTemplate {
    items: Items,
    header_name: String,
}

pub(super) fn compile(context: &mut Context, options: Compile, items: Items) -> Result<(), ()> {
    let file_stem = Path::new(&options.file)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let output_dir = options.resolve_output_dir();

    let header_filename = format!("{}.hpp", file_stem);
    let header_path = output_path(output_dir.as_ref(), &header_filename);
    if let Err(e) = compile_header(items.clone(), &header_path) {
        context.report(
            &Diagnostic::error()
                .with_message(format!("error writing to `{}`", header_path.display()))
                .with_notes(vec![format!("{}", e)]),
        );
        return Err(());
    }

    if options.mode == Mode::Export {
        let inline_filename = format!("{}_export.inl", file_stem);
        let inline_path = output_path(output_dir.as_ref(), &inline_filename);
        if let Err(e) = compile_export_inline(items, header_filename, &inline_path) {
            context.report(
                &Diagnostic::error()
                    .with_message(format!("error writing to `{}`", inline_path.display()))
                    .with_notes(vec![format!("{}", e)]),
            );
            return Err(());
        }
    }

    Ok(())
}

fn output_path(output_dir: Option<&PathBuf>, file_name: &str) -> PathBuf {
    if let Some(dir) = output_dir {
        dir.join(file_name)
    } else {
        PathBuf::from(file_name)
    }
}

fn compile_header(items: Items, output_file: &Path) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)?;

    let template = CppHeaderTemplate { items };
    write!(file, "{}", template.render().unwrap())?;
    Ok(())
}

fn compile_export_inline(items: Items, header_name: String, output_file: &Path) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)?;

    let template = CppExportInlineTemplate { items, header_name };
    write!(file, "{}", template.render().unwrap())?;
    Ok(())
}

struct DisplayTypeAbi<'a>(&'a ast::Type);
struct DisplayTypeRaii<'a>(&'a ast::Type);

impl std::fmt::Display for DisplayTypeAbi<'_> {
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
                write!(f, "{}{} *", DisplayTypeAbi(p.ty.as_ref()), modifier)?;
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
                    "vellum::detail::abi::slice<{}{}>",
                    modifier,
                    DisplayTypeAbi(s.ty.as_ref())
                )?;
            }
            ast::Type::Owned(p) => {
                write!(f, "vellum::detail::abi::owned<{}>", DisplayTypeAbi(&p.ty))?;
            }
            ast::Type::FunctionPointer(ast::FunctionPointer {
                fn_ty,
                args,
                returns,
                ..
            }) => {
                let fn_ty_name = match fn_ty {
                    ast::FunctionType::Function => "function",
                    ast::FunctionType::Closure => "detail::abi::closure",
                };
                let fn_ret_ty = if let Some(returns) = &returns {
                    DisplayTypeAbi(returns).to_string()
                } else {
                    "void".to_string()
                };

                write!(f, "vellum::{}<{} (", fn_ty_name, fn_ret_ty,)?;
                if !args.is_empty() {
                    for arg in args.iter().take(args.len() - 1) {
                        write!(f, "{}, ", DisplayTypeAbi(&arg.1))?;
                    }
                    write!(f, "{}", DisplayTypeAbi(&args.last().unwrap().1))?;
                }
                write!(f, ")>")?;
            }
            ast::Type::Array(a) => {
                write!(f, "std::array<{}, {}>", DisplayTypeAbi(&a.ty), a.len)?;
            }
            ast::Type::Identifier(i) => write!(f, "{}", i.identifier)?,
        }
        Ok(())
    }
}

impl std::fmt::Display for DisplayTypeRaii<'_> {
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
                write!(f, "{}{} *", DisplayTypeRaii(p.ty.as_ref()), modifier)?;
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
                    DisplayTypeRaii(s.ty.as_ref())
                )?;
            }
            ast::Type::Owned(p) => {
                write!(f, "vellum::owned<{}>", DisplayTypeRaii(&p.ty))?;
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
                let fn_ret_ty = if let Some(returns) = &returns {
                    DisplayTypeRaii(returns).to_string()
                } else {
                    "void".to_string()
                };

                write!(f, "vellum::{}<{} (", fn_ty_name, fn_ret_ty,)?;
                if !args.is_empty() {
                    for arg in args.iter().take(args.len() - 1) {
                        write!(f, "{}, ", DisplayTypeRaii(&arg.1))?;
                    }
                    write!(f, "{}", DisplayTypeRaii(&args.last().unwrap().1))?;
                }
                write!(f, ")>")?;
            }
            ast::Type::Array(a) => {
                write!(f, "std::array<{}, {}>", DisplayTypeRaii(&a.ty), a.len)?;
            }
            ast::Type::Identifier(i) => write!(f, "{}", i.identifier)?,
        }
        Ok(())
    }
}

mod filters {
    use super::*;

    pub fn ty(ty: &ast::Type, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(DisplayTypeAbi(ty).to_string())
    }

    pub fn retty(ty: &Option<ast::Type>, _: &dyn askama::Values) -> askama::Result<String> {
        if let Some(ty) = ty {
            Ok(DisplayTypeAbi(ty).to_string())
        } else {
            return Ok("void".to_string());
        }
    }

    pub fn ty_raii(ty: &ast::Type, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(DisplayTypeRaii(ty).to_string())
    }

    pub fn retty_raii(ty: &Option<ast::Type>, _: &dyn askama::Values) -> askama::Result<String> {
        if let Some(ty) = ty {
            Ok(DisplayTypeRaii(ty).to_string())
        } else {
            return Ok("void".to_string());
        }
    }
}
