use clap::{Parser, Subcommand};

mod compile;
mod parse;

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
    let args = Args::parse();

    match args.action {
        Action::Compile(compile) => compile::compile(compile),
    }
}
