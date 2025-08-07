use crate::parse::{Context, ast};
use codespan_reporting::diagnostic::{Diagnostic, Label};
use std::collections::HashMap;

/// Append any items to this list that affect the layout of the queried type
fn add_layout_deps(ty: &ast::Type, mut deps: &mut Vec<String>) {
    match &ty {
        ast::Type::Primitive {
            location: _,
            primitive: _,
        } => {}
        ast::Type::Pointer(_) => {}
        ast::Type::String(_) => {}
        ast::Type::Slice(_) => {}
        ast::Type::Owned(_) => {}
        ast::Type::FunctionPointer(_) => {}
        ast::Type::Array(a) => add_layout_deps(a.ty.as_ref(), &mut deps),
        ast::Type::Identifier(ident) => deps.push(ident.identifier.clone()),
    }
}

/// Check basic type properties
fn type_checks(context: &mut Context, items: &HashMap<String, ast::Item>) -> Result<(), ()> {
    // Flatten the type tree
    let mut types = Vec::new();
    for (_, item) in items {
        match &item.item {
            ast::ItemType::Import(_) => unreachable!("imports should have been resolved"),
            ast::ItemType::Struct(s) => {
                if let Some(fields) = &s.fields {
                    for field in fields.iter() {
                        types.extend(field.ty.iter_tree());
                    }
                }
            }
            ast::ItemType::Function(f) => {
                for (_, ty) in f.args.iter() {
                    types.extend(ty.iter_tree());
                }
                types.extend(f.returns.iter_tree())
            }
        }
    }
    let types = types;

    // Check idents
    let mut bad_ident = false;
    for ty in &types {
        if let ast::Type::Identifier(ident) = ty {
            if let Some(item) = items.get(&ident.identifier) {
                let bad_item = match &item.item {
                    ast::ItemType::Struct(_) => None,
                    ast::ItemType::Import(_) => unreachable!("imports should have been resolved"),
                    ast::ItemType::Function(f) => Some(("function", f.location.clone())),
                };
                if let Some((bad_item_name, bad_item_loc)) = bad_item {
                    context.report(
                        &Diagnostic::error()
                            .with_message("expected type")
                            .with_labels(vec![
                                Label::primary(ident.location.file_id, ident.location.span.clone())
                                    .with_message(format!("got a {}", bad_item_name)),
                                Label::secondary(bad_item_loc.file_id, bad_item_loc.span.clone())
                                    .with_message("defined here"),
                            ]),
                    );
                    bad_ident = true;
                }
            } else {
                context.report(
                    &Diagnostic::error()
                        .with_message(format!("no type `{}` found", ident.identifier))
                        .with_labels(vec![
                            Label::primary(ident.location.file_id, ident.location.span.clone())
                                .with_message("used here"),
                        ]),
                );
                bad_ident = true;
            }
        }
    }
    if bad_ident {
        return Err(());
    }

    // Check proper sizedness of all types
    let mut bad_sized = false;
    for ty in &types {
        match &ty {
            ast::Type::Primitive {
                location: _,
                primitive: _,
            } => {}
            ast::Type::Pointer(_) => {}
            ast::Type::String(_) => {}
            ast::Type::Slice(_) => {}
            ast::Type::Owned(_) => {}
            ast::Type::FunctionPointer(_) => {}
            ast::Type::Array(a) => {
                if !is_sized(a.ty.as_ref(), &items) {
                    context.report(
                        &Diagnostic::error()
                            .with_message("array element must be a sized type")
                            .with_labels(vec![Label::primary(
                                a.location.file_id,
                                a.location.span.clone(),
                            )]),
                    );
                    bad_sized = true;
                }
            }
            ast::Type::Identifier(_) => {}
        }
    }
    if bad_sized {
        return Err(());
    }

    Ok(())
}

/// Returns if the type is sized.
///
/// A type is sized if we know how big it is. This is the same as Rust Sized, or C/C++ complete.
fn is_sized(ty: &ast::Type, items: &HashMap<String, ast::Item>) -> bool {
    match &ty {
        ast::Type::Primitive {
            location: _,
            primitive: _,
        } => true,
        ast::Type::Pointer(_) => true,
        ast::Type::String(_) => true,
        ast::Type::Slice(_) => true,
        ast::Type::Owned(_) => true,
        ast::Type::FunctionPointer(_) => true,
        ast::Type::Array(_) => true,
        ast::Type::Identifier(ident) => {
            if let Some(item) = items.get(&ident.identifier) {
                match &item.item {
                    ast::ItemType::Struct(s) => s.fields.is_some(),
                    _ => unreachable!("ident check should have caught this"),
                }
            } else {
                false
            }
        }
    }
}

/// Assert that types only reference concrete types by value.
///
/// Abstract types can be referenced only by pointer.
///
/// Returns concrete type dependencies.
pub fn check(
    context: &mut Context,
    items: &HashMap<String, ast::Item>,
) -> Result<HashMap<String, Vec<String>>, ()> {
    type_checks(context, items)?;

    let mut dependencies = HashMap::new();

    for (name, item) in items {
        match &item.item {
            ast::ItemType::Import(_) => unimplemented!(),
            ast::ItemType::Struct(s) => {
                if let Some(fields) = &s.fields {
                    // Check the following:
                    // * Fields must have unique names
                    // * Field types must be sized
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

                        if !is_sized(&field.ty, &items) {
                            context.report(
                                &Diagnostic::error()
                                    .with_message("field is not a sized type")
                                    .with_labels(vec![
                                        Label::primary(
                                            field.ty.location().file_id,
                                            field.ty.location().span.clone(),
                                        )
                                        .with_message("structs without fields are not sized, but can be referenced through pointers"),
                                    ]),
                            );
                        }

                        add_layout_deps(&field.ty, &mut these_dependencies);
                    }
                    dependencies.insert(name.clone(), these_dependencies);
                }
            }
            ast::ItemType::Function(f) => {
                // Check the following:
                // * Arguments must have unique names
                // * Argument types must be sized
                let mut visited_args = HashMap::new();
                let mut these_dependencies = Vec::new();

                for (name, ty) in &f.args {
                    // Check that the field name is unique
                    if let Some(existing_field) = visited_args.insert(&name.identifier, name) {
                        context.report(
                            &Diagnostic::error()
                                .with_message("argument name must be unique")
                                .with_labels(vec![
                                    Label::primary(
                                        name.location.file_id,
                                        name.location.span.clone(),
                                    )
                                    .with_message("duplicate argument name"),
                                    Label::secondary(
                                        existing_field.location.file_id,
                                        existing_field.location.span.clone(),
                                    )
                                    .with_message("first used here"),
                                ]),
                        );
                    }

                    if !is_sized(&ty, &items) {
                        context.report(
                            &Diagnostic::error()
                                .with_message("argument is not a sized type")
                                .with_labels(vec![
                                    Label::primary(
                                        ty.location().file_id,
                                        ty.location().span.clone(),
                                    )
                                    .with_message("structs without fields are not sized, but can be referenced through pointers"),
                                ]),
                        );
                    }

                    add_layout_deps(&ty, &mut these_dependencies);
                }

                add_layout_deps(&f.returns, &mut these_dependencies);

                if !is_sized(&f.returns, &items) {
                    context.report(
                        &Diagnostic::error()
                            .with_message("return type is not a sized type")
                            .with_labels(vec![
                                Label::primary(
                                    f.returns.location().file_id,
                                    f.returns.location().span.clone(),
                                )
                                .with_message("structs without fields are not sized, but can be referenced through pointers"),
                            ]),
                    );
                }

                dependencies.insert(name.clone(), these_dependencies);
            }
        }
    }
    Ok(dependencies)
}
