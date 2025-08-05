use crate::parse::ast;
use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};

mod cpp;

#[derive(ValueEnum, Copy, Clone)]
enum Language {
    Cpp,
}

#[derive(Parser)]
pub struct Compile {
    #[clap(value_enum)]
    language: Language,

    file: String,

    #[clap(short = 'o')]
    output_dir: Option<String>,
}

impl Compile {
    fn resolve_output_dir(&self) -> Option<PathBuf> {
        self.output_dir
            .as_ref()
            .map(|p| Path::new(&p).to_path_buf())
            .or_else(|| Path::new(&self.file).parent().map(Path::to_path_buf))
    }
}

pub(crate) struct AbstractStruct {
    docs: Vec<String>,
    name: String,
}

pub(crate) struct Field {
    docs: Vec<String>,
    name: String,
    ty: ast::Type,
}

pub(crate) struct Struct {
    docs: Vec<String>,
    name: String,
    fields: Vec<Field>,
}

pub(crate) struct Function {
    docs: Vec<String>,
    name: String,
    args: Vec<(String, ast::Type)>,
    returns: ast::Type,
}

pub(crate) struct Items {
    abstract_structs: Vec<AbstractStruct>,
    structs: Vec<Struct>,
    functions: Vec<Function>,
}

pub fn compile(compile: Compile) -> Result<(), ()> {
    let (mut context, file) = crate::parse::parse_program(&compile.file)?;
    let items = crate::type_check::type_check(&mut context, file)?;
    let abstract_structs = items
        .iter()
        .filter_map(|i| match &i.item {
            ast::ItemType::Struct(s) if s.fields.is_none() => Some(AbstractStruct {
                docs: i.docs.clone(),
                name: s.name.identifier.clone(),
            }),
            _ => None,
        })
        .collect();
    let structs = items
        .iter()
        .filter_map(|i| match &i.item {
            ast::ItemType::Struct(s) if s.fields.is_some() => Some(Struct {
                docs: i.docs.clone(),
                name: s.name.identifier.clone(),
                fields: s
                    .fields
                    .as_ref()
                    .unwrap()
                    .iter()
                    .map(|field| Field {
                        docs: field.docs.clone(),
                        name: field.name.identifier.clone(),
                        ty: field.ty.clone(),
                    })
                    .collect(),
            }),
            _ => None,
        })
        .collect();
    let functions = items
        .iter()
        .filter_map(|i| match &i.item {
            ast::ItemType::Function(f) => Some(Function {
                docs: i.docs.clone(),
                name: f.name.identifier.clone(),
                args: f
                    .args
                    .iter()
                    .map(|(name, ty)| (name.identifier.clone(), ty.clone()))
                    .collect(),
                returns: f.returns.as_ref().clone(),
            }),
            _ => None,
        })
        .collect();
    let items = Items {
        abstract_structs,
        structs,
        functions,
    };
    match compile.language {
        Language::Cpp => cpp::compile(&mut context, compile, items),
    }
}
