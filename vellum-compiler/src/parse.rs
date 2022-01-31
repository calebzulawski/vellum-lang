use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        emit,
        termcolor::{BufferedStandardStream, ColorChoice, WriteColor},
        Config,
    },
};
use lalrpop_util::{lalrpop_mod, ParseError};
use std::path::Path;

pub mod ast;

mod lexer;

pub use lexer::Token;

lalrpop_mod!(
    #[allow(clippy::all)]
    grammar,
    "/parse/grammar.rs"
);

pub struct Context {
    files: SimpleFiles<String, String>,
    output: Box<dyn WriteColor>,
    config: Config,
}

impl Context {
    fn new() -> Self {
        Self {
            files: SimpleFiles::new(),
            output: Box::new(BufferedStandardStream::stderr(ColorChoice::Auto)),
            config: Default::default(),
        }
    }

    pub fn report(&mut self, diagnostic: &Diagnostic<usize>) {
        emit(&mut self.output, &self.config, &self.files, diagnostic).unwrap()
    }

    fn add_source(
        &mut self,
        file: &Path,
        location: Option<ast::Location>,
    ) -> Result<(usize, &str), ()> {
        let source = std::fs::read_to_string(file).map_err(|e| {
            let error = Diagnostic::error()
                .with_message(format!("couldn't open `{}`", file.display()))
                .with_notes(vec![format!("{}", e)]);
            let error = if let Some(location) = location {
                error.with_labels(vec![Label::primary(location.file_id, location.span)])
            } else {
                error
            };
            self.report(&error);
        })?;
        let file_id = self.files.add(file.display().to_string(), source);
        Ok((file_id, self.files.get(file_id).unwrap().source()))
    }

    fn parse_file(
        &mut self,
        file: &Path,
        location: Option<ast::Location>,
    ) -> Result<Vec<ast::Item>, ()> {
        let (file_id, source) = self.add_source(file, location)?;
        let lexer = lexer::Lexer::new(source);
        grammar::ProgramParser::new()
            .parse(file_id, lexer)
            .map_err(|e| match e {
                ParseError::InvalidToken { location } => {
                    self.report(
                        &Diagnostic::error()
                            .with_message("could not parse")
                            .with_labels(vec![Label::primary(file_id, location..location)]),
                    );
                }
                ParseError::UnrecognizedEOF { location, expected } => {
                    let expected = expected.join(", ");
                    self.report(
                        &Diagnostic::error()
                            .with_message("reached end of file")
                            .with_labels(vec![Label::primary(file_id, location..location)])
                            .with_notes(vec![format!("expected one of: {}", expected)]),
                    );
                }
                ParseError::UnrecognizedToken {
                    token: (left, _, right),
                    expected,
                } => {
                    let expected = expected.join(", ");
                    self.report(
                        &Diagnostic::error()
                            .with_message("unexpected token")
                            .with_labels(vec![Label::primary(file_id, left..right)])
                            .with_notes(vec![format!("expected one of: {}", expected)]),
                    );
                }
                ParseError::ExtraToken {
                    token: (left, _, right),
                } => {
                    self.report(
                        &Diagnostic::error()
                            .with_message("unexpected token")
                            .with_labels(vec![Label::primary(file_id, left..right)]),
                    );
                }
                e => panic!("unexpected lexer error: {:?}", e),
            })
    }
}

pub fn parse_program(file: impl AsRef<Path>) -> Result<(Context, Vec<ast::Item>), ()> {
    // TODO replace with iterators, with fewer allocations
    fn handle_imports(
        context: &mut Context,
        mut items: Vec<ast::Item>,
        file: &Path,
    ) -> Result<Vec<ast::Item>, ()> {
        let mut new_items = Vec::new();
        for item in items.drain(..) {
            if let ast::Item {
                docs: _,
                item: ast::ItemType::Import(ast::Import { location, path }),
            } = item
            {
                let next_path = if let Some(parent) = file.parent() {
                    parent.join(path)
                } else {
                    Path::new(&path).to_owned()
                };
                let next_items = context.parse_file(&next_path, Some(location))?;
                new_items.append(&mut handle_imports(context, next_items, &next_path)?);
            } else {
                new_items.push(item);
            }
        }
        Ok(new_items)
    }

    let mut context = Context::new();
    let items = context.parse_file(file.as_ref(), None)?;
    let items = handle_imports(&mut context, items, file.as_ref())?;
    Ok((context, items))
}
