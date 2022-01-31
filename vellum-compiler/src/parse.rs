use lalrpop_util::{lalrpop_mod, ParseError};

pub mod ast;

mod lexer;

pub use lexer::Token;

lalrpop_mod!(grammar, "/parse/grammar.rs");

pub fn parse_program(input: &str) -> Result<Vec<ast::Item>, ParseError<usize, Token, ()>> {
    let lexer = lexer::Lexer::new(input);
    grammar::ProgramParser::new().parse(0, lexer)
}
