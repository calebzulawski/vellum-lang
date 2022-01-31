use clap::Parser;

#[derive(Parser)]
pub struct Compile {
    file: String,
}

pub fn compile(compile: Compile) {
    crate::parse::parse_program(compile.file).unwrap();
}
