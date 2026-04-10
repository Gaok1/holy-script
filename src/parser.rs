use crate::ast::*;
use crate::lexer::{token_name, Spanned, Token};

mod declarations;
mod expressions;
mod statements;
mod utils;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl ParseError {
    fn at(message: impl Into<String>, line: usize, col: usize) -> Self {
        ParseError { message: message.into(), line, col }
    }
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}, column {}: {}", self.line, self.col, self.message)
    }
}

pub struct Parser {
    tokens: Vec<Spanned>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Spanned>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // ── Token stream primitives ───────────────────────────────────────────────

    fn sp(&self) -> &Spanned {
        self.tokens.get(self.pos).unwrap_or_else(|| self.tokens.last().unwrap())
    }

    fn peek(&self) -> &Token {
        &self.sp().token
    }

    fn advance(&mut self) -> Spanned {
        let sp = self.tokens.get(self.pos)
            .cloned()
            .unwrap_or_else(|| self.tokens.last().unwrap().clone());
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        sp
    }

    fn expect(&mut self, expected: &Token) -> Result<Spanned, ParseError> {
        let sp = self.sp().clone();
        if &sp.token == expected {
            Ok(self.advance())
        } else {
            Err(ParseError::at(expect_msg(expected, &sp.token), sp.line, sp.col))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let sp = self.sp().clone();
        match self.advance().token {
            Token::Ident(name) => Ok(name),
            t => Err(ParseError::at(
                format!("a sacred name was demanded, but '{}' was offered in its place", token_name(&t)),
                sp.line,
                sp.col,
            )),
        }
    }

    fn expect_variant_name(&mut self) -> Result<String, ParseError> {
        self.expect_ident()
    }

    fn parse_builtin_covenant_name(&mut self) -> Result<String, ParseError> {
        self.expect_ident()
    }

    // ── Program entry point ───────────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut testaments = Vec::new();
        let mut top_decls  = Vec::new();
        let mut stmts      = Vec::new();

        while self.peek() == &Token::Testament {
            testaments.push(self.parse_testament()?);
        }

        while matches!(self.peek(), Token::Scripture | Token::Sin | Token::Covenant | Token::Salm) {
            top_decls.push(self.parse_top_decl()?);
        }

        while !matches!(self.peek(), Token::Amen | Token::Eof) {
            stmts.push(self.parse_stmt()?);
        }

        self.expect(&Token::Amen)?;

        Ok(Program { testaments, top_decls, stmts })
    }

    fn parse_testament(&mut self) -> Result<Testament, ParseError> {
        self.expect(&Token::Testament)?;

        let name = self.expect_ident()?;

        // Zero or more `from segment` clauses for subdirectory paths
        let mut path = vec![];
        while self.peek() == &Token::From {
            self.advance();
            path.push(self.expect_ident()?);
        }

        let revealing = if self.peek() == &Token::Revealing {
            self.advance();
            Some(self.parse_ident_list()?)
        } else {
            None
        };

        Ok(Testament { name, path, revealing })
    }
}

// ── Error messages ────────────────────────────────────────────────────────────

fn expect_msg(expected: &Token, found: &Token) -> String {
    let f = token_name(found);
    match expected {
        Token::Reveals   => format!("'reveals' must be spoken to declare the salm's return type, yet '{}' was uttered", f),
        Token::Receiving => format!("'receiving' must be spoken to list the parameters, yet '{}' was uttered", f),
        Token::Indent    => format!("an indented block was ordained after this line, yet '{}' stood in its way", f),
        Token::Dedent    => format!("the sacred block was left open — it must be closed, yet '{}' was found", f),
        Token::Of        => format!("'of' must be spoken to declare the type (e.g. x of atom), yet '{}' was uttered", f),
        Token::Be        => format!("'be' must follow the type to assign a value (e.g. let there x of atom be 0), yet '{}' was uttered", f),
        Token::Become    => format!("'become' must be spoken to reassign the variable, yet '{}' was uttered", f),
        Token::For       => format!("'for' must be spoken (as in 'litany for' or 'answer for'), yet '{}' was uttered", f),
        Token::Than      => format!("'than' must complete the comparison, yet '{}' was uttered", f),
        Token::There     => format!("'there' must follow 'let', yet '{}' was uttered", f),
        Token::Upon      => format!("'upon' must be spoken to name the target scripture of the method, yet '{}' was uttered", f),
        Token::Eof       => format!("the scripture was meant to end here, yet '{}' lingers uninvited", f),
        _                => format!("the holy order demands '{}', yet '{}' was spoken", token_name(expected), f),
    }
}
