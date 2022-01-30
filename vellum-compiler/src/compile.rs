use clap::Parser;

#[derive(Parser)]
pub struct Compile {
    file: String,
}

pub fn compile(compile: Compile) {
    let source = std::fs::read_to_string(compile.file).unwrap();
    crate::parse::parse_program(source.as_str()).unwrap();
}
