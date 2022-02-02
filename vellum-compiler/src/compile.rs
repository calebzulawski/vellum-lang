use clap::Parser;

#[derive(Parser)]
pub struct Compile {
    file: String,
}

pub fn compile(compile: Compile) {
    let (mut context, file) = crate::parse::parse_program(compile.file).unwrap();
    let _ = crate::type_check::type_check(&mut context, file).unwrap();
}
