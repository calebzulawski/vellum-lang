use clap::{Parser, Subcommand};

mod compile;
mod parse;
mod type_check;

#[derive(Parser)]
#[clap(name = "vellum")]
#[clap(version)]
#[clap(author)]
#[clap(about)]
struct Args {
    #[clap(subcommand)]
    action: Action,
}

#[derive(Subcommand)]
enum Action {
    Compile(compile::Compile),
}

fn main() {
    if let Err(()) = main_impl() {
        std::process::exit(1);
    }
}

fn main_impl() -> Result<(), ()> {
    let args = Args::parse();

    match args.action {
        Action::Compile(compile) => compile::compile(compile),
    }
}
