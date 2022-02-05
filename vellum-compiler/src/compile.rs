use clap::{ArgEnum, Parser};
use std::path::{Path, PathBuf};

mod cpp;

#[derive(ArgEnum, Copy, Clone)]
enum Language {
    Cpp,
}

#[derive(Parser)]
pub struct Compile {
    #[clap(arg_enum)]
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

pub fn compile(compile: Compile) -> Result<(), ()> {
    let (mut context, file) = crate::parse::parse_program(&compile.file)?;
    let items = crate::type_check::type_check(&mut context, file)?;
    match compile.language {
        Language::Cpp => cpp::compile(&mut context, compile, items),
    }
}
