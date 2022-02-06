use super::Compile;
use crate::parse::{ast, Context};
use codespan_reporting::diagnostic::Diagnostic;
use indoc::writedoc;
use std::{
    fs::{File, OpenOptions},
    io::{Error, Write},
    path::Path,
};

pub fn compile(context: &mut Context, options: Compile, items: Vec<ast::Item>) -> Result<(), ()> {
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

fn compile_impl(items: Vec<ast::Item>, output_file: &Path) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output_file)?;
    writedoc!(
        file,
        r##"
        #include <cstdint>
        #include <cstddef>
        #include <vellum.hpp>

        "##
    )?;

    forward_declarations(&items, &mut file)?;

    struct_definitions(&items, &mut file)?;

    Ok(())
}

fn write_docs(file: &mut File, prefix: &str, docs: &[String]) -> Result<(), Error> {
    if !docs.is_empty() {
        writeln!(file, "{}/*!", prefix)?;
        for doc in docs {
            writeln!(file, "{} *!{}", prefix, doc)?;
        }
        writeln!(file, "{} */", prefix)?;
    }
    Ok(())
}

fn forward_declarations(items: &[ast::Item], file: &mut File) -> Result<(), Error> {
    for item in items {
        if let ast::ItemType::Struct(s) = &item.item {
            write_docs(file, "", &item.docs)?;
            writeln!(file, "struct {};", s.name.identifier)?;
        }
    }
    writeln!(file)?;
    Ok(())
}

fn struct_definitions(items: &[ast::Item], file: &mut File) -> Result<(), Error> {
    for item in items {
        if let ast::ItemType::Struct(ast::Struct {
            name,
            fields: Some(fields),
            ..
        }) = &item.item
        {
            write_docs(file, "", &item.docs)?;
            writeln!(file, "struct {} {{", name.identifier)?;
            for field in fields {
                write_docs(file, "  ", &field.docs)?;
                writeln!(
                    file,
                    "  {} {};",
                    DisplayType(&field.ty),
                    field.name.identifier
                )?;
            }
            writeln!(file, "}};\n")?;
        }
    }
    Ok(())
}

struct DisplayType<'a>(&'a ast::Type);

impl std::fmt::Display for DisplayType<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self.0 {
            ast::Type::Primitive { primitive, .. } => {
                let s = match primitive {
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
                if let ast::PointerModifier::Owned = p.modifier {
                    write!(f, "vellum::owned_ptr<{}>", DisplayType(p.ty.as_ref()))?;
                } else {
                    let modifier = match p.modifier {
                        ast::PointerModifier::Const => " const",
                        ast::PointerModifier::Mut => " ",
                        _ => unimplemented!(),
                    };
                    write!(f, "{}{} *", DisplayType(p.ty.as_ref()), modifier)?;
                }
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
                write!(f, "vellum::{}<{} (", fn_ty_name, DisplayType(returns.as_ref()))?;
                if !args.is_empty() {
                    for arg in args.iter().take(args.len() - 1) {
                        write!(f, "{}, ", DisplayType(&arg.1))?;
                    }
                    write!(f, "{}", DisplayType(&args.last().unwrap().1))?;
                }
                write!(f, ")>")?;
            }
            ast::Type::Identifier(i) => write!(f, "{}", i.identifier)?,
        }
        Ok(())
    }
}
