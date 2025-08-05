use crate::parse::{Context, ast};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use std::collections::HashMap;

/// Assert that types only reference concrete types by value.
///
/// Abstract types can be referenced only by pointer.
///
/// Returns concrete type dependencies.
pub fn check(
    context: &mut Context,
    items: &HashMap<String, ast::Item>,
) -> Result<HashMap<String, Vec<String>>, ()> {
    let mut dependencies = HashMap::new();

    for (name, item) in items {
        match &item.item {
            ast::ItemType::Import(_) => unimplemented!(),
            ast::ItemType::Struct(s) => {
                if let Some(fields) = &s.fields {
                    // Check the following:
                    // * Fields must have unique names
                    // * Fields must be concrete types
                    let mut visited_fields = HashMap::new();
                    let mut these_dependencies = Vec::new();
                    for field in fields {
                        // Check that the field name is unique
                        if let Some(existing_field) =
                            visited_fields.insert(&field.name.identifier, &field.name)
                        {
                            context.report(
                                &Diagnostic::error()
                                    .with_message("field names must be unique")
                                    .with_labels(vec![
                                        Label::primary(
                                            field.name.location.file_id,
                                            field.name.location.span.clone(),
                                        )
                                        .with_message("duplicate field name"),
                                        Label::secondary(
                                            existing_field.location.file_id,
                                            existing_field.location.span.clone(),
                                        )
                                        .with_message("first used here"),
                                    ]),
                            );
                        }

                        // Check that the field is concrete
                        check_type(context, items, &field.ty, &mut these_dependencies)?;
                    }
                    dependencies.insert(name.clone(), these_dependencies);
                }
            }
            ast::ItemType::Function(f) => {
                let mut these_dependencies = Vec::new();
                check_function_args(context, items, &f.args, &mut these_dependencies)?;
                check_type(context, items, &f.returns, &mut these_dependencies)?;
                dependencies.insert(name.clone(), these_dependencies);
            }
        }
    }
    Ok(dependencies)
}

fn check_function_args(
    context: &mut Context,
    items: &HashMap<String, ast::Item>,
    args: &[(ast::Identifier, ast::Type)],
    record_dependency: &mut Vec<String>,
) -> Result<(), ()> {
    // Check the following:
    // * Args must have unique names
    // * Args must be concrete types
    let mut visited_args = HashMap::new();
    for (name, ty) in args {
        // Check that the field name is unique
        if let Some(existing_field) = visited_args.insert(&name.identifier, name) {
            context.report(
                &Diagnostic::error()
                    .with_message("argument name must be unique")
                    .with_labels(vec![
                        Label::primary(name.location.file_id, name.location.span.clone())
                            .with_message("duplicate argument name"),
                        Label::secondary(
                            existing_field.location.file_id,
                            existing_field.location.span.clone(),
                        )
                        .with_message("first used here"),
                    ]),
            );
        }

        // Check that the field is concrete
        check_type(context, items, &ty, record_dependency)?;
    }
    Ok(())
}

fn check_type(
    context: &mut Context,
    items: &HashMap<String, ast::Item>,
    ty: &ast::Type,
    record_dependency: &mut Vec<String>,
) -> Result<(), ()> {
    fn check_type(
        context: &mut Context,
        items: &HashMap<String, ast::Item>,
        ty: &ast::Type,
        record_dependency: &mut Vec<String>,
        require_concrete: bool,
    ) -> Result<(), ()> {
        match ty {
            ast::Type::Primitive {
                location: _,
                primitive: _,
            } => Ok(()),
            ast::Type::Pointer(p) => check_type(context, items, &p.ty, record_dependency, false),
            ast::Type::String(_) => Ok(()),
            ast::Type::Owned(p) => {
                check_type(context, items, &p.ty, record_dependency, require_concrete)
            }
            ast::Type::FunctionPointer(f) => {
                check_function_args(context, items, &f.args, record_dependency)?;
                check_type(context, items, &f.returns, record_dependency, true)?;
                Ok(())
            }
            ast::Type::Identifier(ident) => {
                if let Some(item) = items.get(&ident.identifier) {
                    match &item.item {
                        ast::ItemType::Import(_) => unimplemented!(),
                        ast::ItemType::Struct(s) => {
                            if require_concrete && s.fields.is_none() {
                                context.report(
                                    &Diagnostic::error()
                                        .with_message("expected a concrete type")
                                        .with_labels(vec![
                                            Label::primary(
                                                ident.location.file_id,
                                                ident.location.span.clone(),
                                            )
                                            .with_message(format!(
                                                "got abstract type `{}`",
                                                ident.identifier
                                            )),
                                        ]),
                                );
                                Err(())
                            } else if require_concrete {
                                record_dependency.push(ident.identifier.clone());
                                Ok(())
                            } else {
                                Ok(())
                            }
                        }
                        ast::ItemType::Function(f) => {
                            context.report(
                                &Diagnostic::error()
                                    .with_message("expected type")
                                    .with_labels(vec![
                                        Label::primary(
                                            ident.location.file_id,
                                            ident.location.span.clone(),
                                        )
                                        .with_message("got a function"),
                                        Label::secondary(
                                            f.location.file_id,
                                            f.location.span.clone(),
                                        )
                                        .with_message("defined here"),
                                    ]),
                            );
                            Err(())
                        }
                    }
                } else {
                    context.report(
                        &Diagnostic::error()
                            .with_message(format!("undefined type `{}`", ident.identifier))
                            .with_labels(vec![
                                Label::primary(ident.location.file_id, ident.location.span.clone())
                                    .with_message("used here"),
                            ]),
                    );
                    Err(())
                }
            }
        }
    }

    check_type(context, items, ty, record_dependency, true)
}
