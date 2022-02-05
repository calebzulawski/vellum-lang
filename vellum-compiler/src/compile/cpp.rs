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

fn forward_declarations(items: &[ast::Item], file: &mut File) -> Result<(), Error> {
    for item in items {
        if let ast::ItemType::Struct(s) = &item.item {
            writeln!(file, "struct {};", s.name.identifier)?
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
            writeln!(file, "struct {} {{", name.identifier)?;
            for field in fields {
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
            ast::Type::Identifier(i) => write!(f, "{}", i.identifier)?,
        }
        Ok(())
    }
}
