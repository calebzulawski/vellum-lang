use crate::parse::ast;
use clap::{ValueEnum, Parser};

#[derive(ValueEnum, Copy, Clone)]
enum Format {
    List,
    GnuVersionScript,
}

#[derive(Parser)]
pub struct Symbols {
    #[clap(value_enum, short, long, default_value_t = Format::List)]
    format: Format,

    file: String,
}

pub fn symbols(symbols: Symbols) -> Result<(), ()> {
    let (mut context, file) = crate::parse::parse_program(&symbols.file)?;
    let items = crate::type_check::type_check(&mut context, file)?;

    match symbols.format {
        Format::List => list(&items),
        Format::GnuVersionScript => gnu(&items),
    }

    Ok(())
}

fn list(items: &[ast::Item]) {
    for item in items {
        if let ast::ItemType::Function(f) = &item.item {
            println!("{}", f.name.identifier);
        }
    }
}

fn gnu(items: &[ast::Item]) {
    println!("{{");
    println!("  global:");
    for item in items {
        if let ast::ItemType::Function(f) = &item.item {
            println!("    {};", f.name.identifier);
        }
    }
    println!("  local: *;");
    println!("}};");
}
