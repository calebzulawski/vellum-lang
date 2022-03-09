use logos::Logos;

#[derive(Clone, Debug, PartialEq)]
pub enum Primitive {
    I8,
    I16,
    I32,
    I64,
    Isize,
    U8,
    U16,
    U32,
    U64,
    Usize,
}

#[derive(Logos, Clone, Debug, PartialEq)]
pub enum Token {
    #[token("{")]
    LeftBracket,

    #[token("}")]
    RightBracket,

    #[token("(")]
    LeftParen,

    #[token(")")]
    RightParen,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token(";")]
    Semicolon,

    #[token("*")]
    Asterisk,

    #[token("u8", |_| Primitive::U8)]
    #[token("u16", |_| Primitive::U16)]
    #[token("u32", |_| Primitive::U32)]
    #[token("u64", |_| Primitive::U64)]
    #[token("usize", |_| Primitive::Usize)]
    #[token("i8", |_| Primitive::I8)]
    #[token("i16", |_| Primitive::I16)]
    #[token("i32", |_| Primitive::I32)]
    #[token("i64", |_| Primitive::I64)]
    #[token("isize", |_| Primitive::Isize)]
    Primitive(Primitive),

    #[token("struct")]
    Struct,

    #[token("const")]
    Const,

    #[token("owned")]
    Owned,

    #[token("mut")]
    Mut,

    #[token("string")]
    String,

    #[token("import")]
    Import,

    #[token("function")]
    Function,

    #[token("closure")]
    Closure,

    #[token("->")]
    Arrow,

    #[regex(r"'[^\n\r']*'", |lex| {
        let len = lex.slice().len();
        lex.slice()[1..len-1].to_string()
    })]
    StringLiteral(String),

    #[regex(r"[a-zA-Z][a-zA-Z_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    #[regex(r"//[^\n\r]*[\n\r]*", |lex| lex.slice().to_string())]
    Comment(String),

    #[regex(r"///[^\n\r]*[\n\r]*", |lex| lex.slice().to_string())]
    DocComment(String),

    #[regex(r"[ \n\t\f]", logos::skip)]
    #[error]
    Error,
}

pub struct Lexer<'input> {
    lexer: logos::Lexer<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            lexer: logos::Lexer::new(input),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Result<(usize, Token, usize), ()>;

    fn next(&mut self) -> Option<Self::Item> {
        self.lexer
            .next()
            .map(|token| Ok((self.lexer.span().start, token, self.lexer.span().end)))
    }
}
