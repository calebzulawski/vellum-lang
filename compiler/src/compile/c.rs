use super::{Compile, Items, Mode};
use crate::parse::{Context, ast};
use askama::Template;
use codespan_reporting::diagnostic::Diagnostic;
use std::{
    collections::BTreeSet,
    fs::OpenOptions,
    io::{Error, Write},
    path::{Path, PathBuf},
};

// C backend: generates concrete C typedefs and prototypes.
// Differs from C++ (templates/RAII) and Python (ctypes) by materializing
// per-type slice/owned typedefs because C has no templates.
#[derive(Template)]
#[template(path = "c/import.h", escape = "none")]
struct CHeaderTemplate {
    items: Items,
    slice_decls: Vec<SliceDecl>,
    owned_ptr_decls: Vec<OwnedPtrDecl>,
    owned_slice_decls: Vec<OwnedSliceDecl>,
}

#[derive(Template)]
#[template(path = "c/export.inl", escape = "none")]
struct CExportInlineTemplate {
    items: Items,
    header_name: String,
}

// Concrete typedefs needed in emitted C header
#[derive(Clone)]
struct SliceDecl {
    name: String,
    elem_c_type: String,
}
#[derive(Clone)]
struct OwnedPtrDecl {
    name: String,
    data_c_type: String,
}
#[derive(Clone)]
struct OwnedSliceDecl {
    name: String,
    slice_name: String,
}

pub(super) fn compile(context: &mut Context, options: Compile, items: Items) -> Result<(), ()> {
    let file_stem = Path::new(&options.file)
        .file_stem()
        .unwrap()
        .to_string_lossy()
        .into_owned();
    let output_dir = options.resolve_output_dir();

    let header_filename = format!("{}.h", file_stem);
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
    let (slice_decls, owned_ptr_decls, owned_slice_decls) = collect_type_decls(&items);
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)?;

    let template = CHeaderTemplate {
        items,
        slice_decls,
        owned_ptr_decls,
        owned_slice_decls,
    };
    write!(file, "{}", template.render().unwrap())?;
    Ok(())
}

fn compile_export_inline(
    items: Items,
    header_name: String,
    output_file: &Path,
) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)?;

    let template = CExportInlineTemplate { items, header_name };
    write!(file, "{}", template.render().unwrap())?;
    Ok(())
}

// Gather all slice/owned types reachable in the ABI and produce concrete
// typedef names for the C header.
fn collect_type_decls(items: &Items) -> (Vec<SliceDecl>, Vec<OwnedPtrDecl>, Vec<OwnedSliceDecl>) {
    struct Collector {
        slice: BTreeSet<(String, String)>,
        owned_ptr: BTreeSet<(String, String)>,
        owned_slice: BTreeSet<(String, String)>,
    }
    impl Collector {
        fn new() -> Self {
            Self {
                slice: BTreeSet::new(),
                owned_ptr: BTreeSet::new(),
                owned_slice: BTreeSet::new(),
            }
        }
        fn add(&mut self, ty: &ast::Type) {
            match ty {
                ast::Type::Slice(s) => {
                    let name = format!(
                        "{}_{}",
                        if matches!(s.modifier, ast::PointerModifier::Const) {
                            "const"
                        } else {
                            "mut"
                        },
                        mangle_type(&s.ty)
                    );
                    let elem = c_type(&s.ty, matches!(s.modifier, ast::PointerModifier::Const));
                    self.slice.insert((name, elem));
                }
                ast::Type::Owned(o) => {
                    match o.ty.as_ref() {
                        ast::Type::Slice(s) => {
                            let name = format!(
                                "{}_{}",
                                if matches!(s.modifier, ast::PointerModifier::Const) {
                                    "const"
                                } else {
                                    "mut"
                                },
                                mangle_type(&s.ty)
                            );
                            let elem =
                                c_type(&s.ty, matches!(s.modifier, ast::PointerModifier::Const));
                            self.slice.insert((name.clone(), elem));
                            self.owned_slice.insert((name, String::new()));
                        }
                        _ => {
                            // owned pointer data type is the underlying C pointer type
                            let data_ty = DisplayTypeC(&o.ty).to_string();
                            let name = format!("{}", mangle_type(&o.ty));
                            self.owned_ptr.insert((name, data_ty));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    fn visit_type_rec(col: &mut Collector, ty: &ast::Type) {
        col.add(ty);
        match ty {
            ast::Type::Pointer(p) => visit_type_rec(col, &p.ty),
            ast::Type::Slice(s) => visit_type_rec(col, &s.ty),
            ast::Type::Owned(o) => visit_type_rec(col, &o.ty),
            ast::Type::Array(a) => visit_type_rec(col, &a.ty),
            ast::Type::FunctionPointer(fp) => {
                if let Some(r) = &fp.returns {
                    visit_type_rec(col, r);
                }
                for (_, t) in &fp.args {
                    visit_type_rec(col, t);
                }
            }
            _ => {}
        }
    }
    let mut col = Collector::new();
    for s in &items.structs {
        for f in &s.fields {
            visit_type_rec(&mut col, &f.ty);
        }
    }
    for f in &items.functions {
        for (_, t) in &f.args {
            visit_type_rec(&mut col, t);
        }
        if let Some(r) = &f.returns {
            visit_type_rec(&mut col, r);
        }
    }
    let slice_decls = col
        .slice
        .into_iter()
        .map(|(name, elem)| SliceDecl {
            name,
            elem_c_type: elem,
        })
        .collect();
    let owned_ptr_decls = col
        .owned_ptr
        .into_iter()
        .map(|(name, data)| OwnedPtrDecl {
            name,
            data_c_type: data,
        })
        .collect();
    let owned_slice_decls = col
        .owned_slice
        .into_iter()
        .map(|(name, _)| OwnedSliceDecl {
            name: name.clone(),
            slice_name: format!("vellum_slice_{}", name),
        })
        .collect();
    (slice_decls, owned_ptr_decls, owned_slice_decls)
}

fn mangle_type(ty: &ast::Type) -> String {
    match ty {
        ast::Type::Primitive { primitive, .. } => match primitive {
            ast::Primitive::Bool => "bool".into(),
            ast::Primitive::I8 => "i8".into(),
            ast::Primitive::I16 => "i16".into(),
            ast::Primitive::I32 => "i32".into(),
            ast::Primitive::I64 => "i64".into(),
            ast::Primitive::Isize => "isize".into(),
            ast::Primitive::U8 => "u8".into(),
            ast::Primitive::U16 => "u16".into(),
            ast::Primitive::U32 => "u32".into(),
            ast::Primitive::U64 => "u64".into(),
            ast::Primitive::Usize => "usize".into(),
        },
        ast::Type::Pointer(p) => {
            let base = mangle_type(&p.ty);
            match p.modifier {
                ast::PointerModifier::Const => format!("const_{}_ptr", base),
                ast::PointerModifier::Mut => format!("{}_ptr", base),
            }
        }
        ast::Type::String(s) => match s.modifier {
            ast::PointerModifier::Const => "const_char_ptr".into(),
            ast::PointerModifier::Mut => "char_ptr".into(),
        },
        ast::Type::Slice(s) => {
            let base = mangle_type(&s.ty);
            match s.modifier {
                ast::PointerModifier::Const => format!("slice_const_{}", base),
                ast::PointerModifier::Mut => format!("slice_{}", base),
            }
        }
        ast::Type::Owned(o) => format!("owned_{}", mangle_type(&o.ty)),
        ast::Type::FunctionPointer(fp) => {
            let mut s = String::from("fn_");
            if let Some(r) = &fp.returns {
                s.push_str(&mangle_type(r));
            } else {
                s.push_str("void");
            }
            s.push_str("_args");
            for a in &fp.args {
                s.push('_');
                s.push_str(&mangle_type(&a.1));
            }
            s
        }
        ast::Type::Array(a) => format!("array_{}_{}", mangle_type(&a.ty), a.len),
        ast::Type::Identifier(i) => i.identifier.clone(),
    }
}

// Print public C types. Key differences vs C++/Python outputs:
// - Slices/owned use concrete typedef names (vellum_slice_... etc.)
// - Pointers spell const on the pointee type per C conventions.
struct DisplayTypeC<'a>(&'a ast::Type);

impl std::fmt::Display for DisplayTypeC<'_> {
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
                let base = DisplayTypeC(p.ty.as_ref()).to_string();
                match p.modifier {
                    ast::PointerModifier::Const => write!(f, "{} const *", base)?,
                    ast::PointerModifier::Mut => write!(f, "{} *", base)?,
                }
            }
            ast::Type::String(s) => match s.modifier {
                ast::PointerModifier::Const => write!(f, "char const *")?,
                ast::PointerModifier::Mut => write!(f, "char *")?,
            },
            ast::Type::Slice(s) => {
                let name = format!(
                    "vellum_slice_{}_{}",
                    if matches!(s.modifier, ast::PointerModifier::Const) {
                        "const"
                    } else {
                        "mut"
                    },
                    mangle_type(&s.ty)
                );
                write!(f, "{}", name)?;
            }
            ast::Type::Owned(o) => match o.ty.as_ref() {
                ast::Type::Slice(s) => {
                    let name = format!(
                        "vellum_owned_slice_{}_{}",
                        if matches!(s.modifier, ast::PointerModifier::Const) {
                            "const"
                        } else {
                            "mut"
                        },
                        mangle_type(&s.ty)
                    );
                    write!(f, "{}", name)?;
                }
                _ => {
                    let name = format!("vellum_owned_ptr_{}", mangle_type(&o.ty));
                    write!(f, "{}", name)?;
                }
            },
            ast::Type::FunctionPointer(ast::FunctionPointer {
                fn_ty,
                args,
                returns,
                ..
            }) => {
                match fn_ty {
                    ast::FunctionType::Function => {
                        write!(
                            f,
                            "{} (*) (",
                            returns
                                .as_ref()
                                .map(|t| DisplayTypeC(t).to_string())
                                .unwrap_or("void".into())
                        )?;
                        for (i, a) in args.iter().enumerate() {
                            if i > 0 {
                                write!(f, ", ")?;
                            }
                            write!(f, "{}", DisplayTypeC(&a.1))?;
                        }
                        write!(f, ")")?;
                    }
                    ast::FunctionType::Closure => {
                        // Placeholder; fields would be emitted if closures are present
                        write!(f, "struct vellum_closure")?;
                    }
                }
            }
            ast::Type::Array(a) => {
                write!(f, "{}[{}]", DisplayTypeC(&a.ty), a.len)?;
            }
            ast::Type::Identifier(i) => write!(f, "struct {}", i.identifier)?,
        }
        Ok(())
    }
}

// Internal helper for typedefs: element type for slices and owned.
// Returns concrete slice typedef for nested slices.
fn c_type(ty: &ast::Type, is_const: bool) -> String {
    match ty {
        ast::Type::Primitive { .. }
        | ast::Type::String(_)
        | ast::Type::Pointer(_)
        | ast::Type::Array(_)
        | ast::Type::Identifier(_) => {
            let base = DisplayTypeC(ty).to_string();
            if is_const {
                format!("const {}", base)
            } else {
                base
            }
        }
        ast::Type::Slice(s) => {
            let name = format!(
                "vellum_slice_{}_{}",
                if matches!(s.modifier, ast::PointerModifier::Const) {
                    "const"
                } else {
                    "mut"
                },
                mangle_type(&s.ty)
            );
            name
        }
        ast::Type::Owned(o) => DisplayTypeC(&o.ty).to_string(),
        // Function pointers are only embedded as opaque pointers here
        ast::Type::FunctionPointer(_) => "void*".into(),
    }
}

mod filters {
    use super::*;

    pub fn ty(ty: &ast::Type, _: &dyn askama::Values) -> askama::Result<String> {
        Ok(DisplayTypeC(ty).to_string())
    }

    pub fn retty(ty: &Option<ast::Type>, _: &dyn askama::Values) -> askama::Result<String> {
        if let Some(ty) = ty {
            Ok(DisplayTypeC(ty).to_string())
        } else {
            Ok("void".into())
        }
    }
}
