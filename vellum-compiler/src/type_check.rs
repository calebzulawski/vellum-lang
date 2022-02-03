use crate::parse::{ast, Context};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use std::collections::HashMap;

mod valid;

fn name_of_item(item: &ast::Item) -> ast::Identifier {
    match &item.item {
        ast::ItemType::Struct(s) => s.name.clone(),
        ast::ItemType::Import(_) => unimplemented!(),
    }
}

fn flatten(context: &mut Context, file: ast::File) -> Result<HashMap<String, ast::Item>, ()> {
    fn flatten(
        context: &mut Context,
        file: ast::File,
        items: &mut HashMap<String, ast::Item>,
    ) -> Result<(), ()> {
        for item in file.items {
            if let ast::ItemType::Import(i) = item.item {
                flatten(context, i.resolved.unwrap(), items)?;
            } else {
                let name = name_of_item(&item);
                if let Some(existing) = items.insert(name.identifier.clone(), item) {
                    let location = name_of_item(&existing).location;
                    context.report(
                        &Diagnostic::error()
                            .with_message(format!("name `{}` already used", name.identifier))
                            .with_labels(vec![
                                Label::primary(location.file_id, location.span)
                                    .with_message("first used here"),
                                Label::secondary(name.location.file_id, name.location.span)
                                    .with_message("used again here"),
                            ]),
                    );
                    return Err(());
                }
            }
        }
        Ok(())
    }

    let mut items = HashMap::new();
    flatten(context, file, &mut items)?;
    Ok(items)
}

pub fn type_check(context: &mut Context, file: ast::File) -> Result<Program, ()> {
    let items = flatten(context, file)?;
    valid::check(context, &items)?;
    Ok(Program {
        structs: Vec::new(),
    })
}

pub struct Program {
    structs: Vec<ast::Struct>,
}
