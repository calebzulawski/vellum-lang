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
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

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
        location: Option<&ast::Location>,
    ) -> Result<(PathBuf, usize, &str), ()> {
        let handle_err =
            |context: &mut Context, file: &Path, location: Option<&ast::Location>, e| {
                let error = Diagnostic::error()
                    .with_message(format!("couldn't open `{}`", file.display()))
                    .with_notes(vec![format!("{}", e)]);
                let error = if let Some(location) = location {
                    error.with_labels(vec![Label::primary(
                        location.file_id,
                        location.span.clone(),
                    )])
                } else {
                    error
                };
                context.report(&error);
            };
        let file = std::fs::canonicalize(file).map_err(|e| handle_err(self, &file, location, e))?;
        let source =
            std::fs::read_to_string(&file).map_err(|e| handle_err(self, &file, location, e))?;
        let file_id = self.files.add(file.display().to_string(), source);
        Ok((file, file_id, self.files.get(file_id).unwrap().source()))
    }

    fn parse_file(
        &mut self,
        path: &Path,
        location: Option<&ast::Location>,
    ) -> Result<ast::File, ()> {
        let (path, file_id, source) = self.add_source(path, location)?;
        let lexer = lexer::Lexer::new(source);
        let items = grammar::ProgramParser::new()
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
            })?;
        Ok(ast::File { path, items })
    }
}

pub fn parse_program(file: impl AsRef<Path>) -> Result<(Context, ast::File), ()> {
    fn parse_file(
        context: &mut Context,
        path: &Path,
        paths: &mut HashMap<PathBuf, Option<ast::Location>>,
        location: Option<&ast::Location>,
    ) -> Result<ast::File, ()> {
        let mut file = context.parse_file(path.as_ref(), location)?;

        // If this file has already been imported, error
        if let Some(first_location) = paths.insert(file.path.clone(), location.cloned()) {
            let error = Diagnostic::error().with_message("file already imported");
            let error = if let Some(first_location) = first_location {
                error.with_labels(vec![
                    Label::primary(location.unwrap().file_id, location.unwrap().span.clone()),
                    Label::secondary(first_location.file_id, first_location.span)
                        .with_message("first imported here"),
                ])
            } else {
                error.with_labels(vec![Label::primary(
                    location.unwrap().file_id,
                    location.unwrap().span.clone(),
                )])
            };
            context.report(&error);
            return Err(());
        }

        // Find and load import items
        for item in file.items.iter_mut() {
            let current_path = path;
            if let ast::Item {
                docs: _,
                item:
                    ast::ItemType::Import(ast::Import {
                        location,
                        path,
                        resolved,
                    }),
            } = item
            {
                assert!(resolved.is_none());
                let next_path = if let Some(parent) = current_path.parent() {
                    parent.join(path)
                } else {
                    Path::new(&path).to_owned()
                };
                *resolved = Some(parse_file(context, &next_path, paths, Some(location))?);
            }
        }
        Ok(file)
    }

    let mut context = Context::new();
    let mut paths = HashMap::new();
    let file = parse_file(&mut context, file.as_ref(), &mut paths, None)?;
    Ok((context, file))
}
