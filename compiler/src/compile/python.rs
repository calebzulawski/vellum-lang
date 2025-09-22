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
#[template(path = "python/module.py", escape = "none")]
struct PyTemplate {
    items: Items,
}

pub(super) fn compile(context: &mut Context, options: Compile, items: Items) -> Result<(), ()> {
    let file_name = Path::new(&options.file)
        .with_extension("py")
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
        .create(true)
        .open(output_file)?;

    let template = PyTemplate { items };
    write!(file, "{}", template.render().unwrap())?;
    Ok(())
}

struct DisplayType<'a>(&'a ast::Type);

impl std::fmt::Display for DisplayType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.0 {
            ast::Type::Primitive { primitive, .. } => {
                let s = match primitive {
                    ast::Primitive::Bool => "ct.c_bool",
                    ast::Primitive::I8 => "ct.c_int8",
                    ast::Primitive::I16 => "ct.c_int16",
                    ast::Primitive::I32 => "ct.c_int32",
                    ast::Primitive::I64 => "ct.c_int64",
                    ast::Primitive::Isize => "ct.c_ssize_t",
                    ast::Primitive::U8 => "ct.c_uint8",
                    ast::Primitive::U16 => "ct.c_uint16",
                    ast::Primitive::U32 => "ct.c_uint32",
                    ast::Primitive::U64 => "ct.c_uint64",
                    ast::Primitive::Usize => "ct.c_size_t",
                };
                write!(f, "{}", s)?;
            }
            ast::Type::Pointer(p) => {
                // ctypes doesn't have constness
                write!(f, "ct.POINTER({})", DisplayType(p.ty.as_ref()))?;
            }
            ast::Type::String(_) => {
                // ctypes doesn't have constness
                write!(f, "ct.c_char_p")?;
            }
            ast::Type::Slice(s) => {
                // ctypes doesn't have constness
                write!(f, "vellum.Slice({})", DisplayType(s.ty.as_ref()))?;
            }
            ast::Type::Owned(p) => {
                // ctypes doesn't have constness
                write!(f, "vellum.Owned({})", DisplayType(p.ty.as_ref()))?;
            }
            ast::Type::FunctionPointer(ast::FunctionPointer {
                fn_ty,
                args,
                returns,
                ..
            }) => {
                let fn_ret_ty = if let Some(returns) = &returns {
                    DisplayType(returns).to_string()
                } else {
                    "None".to_string()
                };
                let fn_ty_name = match fn_ty {
                    ast::FunctionType::Function => "ct.CFUNCTYPE",
                    ast::FunctionType::Closure => "vellum.Closure",
                };

                write!(f, "{}({}", fn_ty_name, fn_ret_ty)?;

                for arg in args.iter() {
                    write!(f, ", {}", DisplayType(&arg.1))?;
                }

                write!(f, ")")?;
            }
            ast::Type::Array(a) => {
                write!(f, "({} * {})", DisplayType(&a.ty), a.len)?;
            }
            ast::Type::Identifier(i) => write!(f, "{}", i.identifier)?,
        }
        Ok(())
    }
}

mod filters {
    use super::*;
    use crate::compile::{Field, Function};

    pub fn ty(ty: &ast::Type, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(DisplayType(ty).to_string())
    }

    pub fn retty(ty: &Option<ast::Type>, _: &dyn askama::Values) -> askama::Result<String> {
        if let Some(ty) = ty {
            Ok(DisplayType(ty).to_string())
        } else {
            return Ok("None".to_string());
        }
    }

    pub fn repr(value: &String, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(format!("{:?}", value))
    }

    pub fn with_incomplete_note(
        docs: &Vec<String>,
        _: &dyn askama::Values,
    ) -> askama::Result<Vec<String>> {
        let mut lines = docs.clone();
        if !lines.is_empty() {
            lines.push(String::new());
        }
        lines.push("Incomplete type; field definitions are provided elsewhere.".to_string());
        Ok(lines)
    }

    pub fn field_docs(
        fields: &Vec<Field>,
        _: &dyn askama::Values,
    ) -> askama::Result<Vec<String>> {
        let mut lines = Vec::new();
        let mut has_any = false;

        for field in fields {
            if field.docs.is_empty() {
                continue;
            }

            if !has_any {
                lines.push("Fields:".to_string());
                has_any = true;
            }

            let mut docs = field.docs.iter();
            if let Some(first) = docs.next() {
                lines.push(format!("- {}: {}", field.name, first));
            }
            lines.extend(docs.map(|doc| format!("  {}", doc)));
        }

        Ok(lines)
    }

    pub fn function_doc_lines(
        functions: &Vec<Function>,
        _: &dyn askama::Values,
    ) -> askama::Result<Vec<String>> {
        let mut lines = Vec::new();

        for function in functions {
            lines.push(String::new());
            lines.push(format!("- {}", function.name));
            lines.extend(function.docs.iter().map(|doc| format!("  {}", doc)));
        }

        Ok(lines)
    }
}
