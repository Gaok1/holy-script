use crate::ast::*;
use crate::lexer::{token_name, Spanned, Token};

mod declarations;
mod expressions;
mod statements;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
    pub line: usize,
    pub col: usize,
}

impl ParseError {
    fn at(message: impl Into<String>, line: usize, col: usize) -> Self {
        ParseError {
            message: message.into(),
            line,
            col,
        }
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

    fn sp(&self) -> &Spanned {
        self.tokens
            .get(self.pos)
            .unwrap_or(self.tokens.last().unwrap())
    }

    fn peek(&self) -> &Token {
        &self.sp().token
    }

    fn advance(&mut self) -> Spanned {
        let sp = self
            .tokens
            .get(self.pos)
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
            Err(ParseError::at(
                expect_msg(expected, &sp.token),
                sp.line,
                sp.col,
            ))
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let sp = self.sp().clone();
        match self.advance().token {
            Token::Ident(name) => Ok(name),
            t => Err(ParseError::at(
                format!("expected an identifier, found {}", token_name(&t)),
                sp.line,
                sp.col,
            )),
        }
    }

    fn expect_variant_name(&mut self) -> Result<String, ParseError> {
        self.expect_ident()
    }

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut testaments = Vec::new();
        let mut top_decls = Vec::new();
        let mut stmts = Vec::new();

        while self.peek() == &Token::Testament {
            testaments.push(self.parse_testament()?);
        }

        while matches!(
            self.peek(),
            Token::Scripture | Token::Sin | Token::Covenant | Token::Salm
        ) {
            top_decls.push(self.parse_top_decl()?);
        }

        while !matches!(self.peek(), Token::Amen | Token::Eof) {
            stmts.push(self.parse_stmt()?);
        }

        self.expect(&Token::Amen)?;

        Ok(Program {
            testaments,
            top_decls,
            stmts,
        })
    }

    fn parse_testament(&mut self) -> Result<Testament, ParseError> {
        self.expect(&Token::Testament)?;
        let name = self.expect_ident()?;

        let revealing = if self.peek() == &Token::Revealing {
            self.advance();
            Some(self.parse_ident_list()?)
        } else {
            None
        };

        Ok(Testament { name, revealing })
    }
}

fn expect_msg(expected: &Token, found: &Token) -> String {
    let f = token_name(found);
    match expected {
        Token::Reveals => format!(
            "expected 'reveals' to declare the salm return type, found {}",
            f
        ),
        Token::Receiving => format!("expected 'receiving' to list parameters, found {}", f),
        Token::Indent => format!("expected an indented block after this line, found {}", f),
        Token::Dedent => format!("block not properly closed, found {}", f),
        Token::Of => format!(
            "expected 'of' to declare the type (e.g. x of atom), found {}",
            f
        ),
        Token::Be => format!(
            "expected 'be' after the type to assign a value (e.g. let there x of atom be 0), found {}",
            f
        ),
        Token::Become => format!("expected 'become' to reassign the variable, found {}", f),
        Token::For => format!(
            "expected 'for' (in 'litany for' or 'answer for'), found {}",
            f
        ),
        Token::Than => format!(
            "expected 'than' to complete the comparison operator, found {}",
            f
        ),
        Token::There => format!("expected 'there' after 'let', found {}", f),
        Token::Upon => format!(
            "expected 'upon' to indicate the target scripture of the method, found {}",
            f
        ),
        Token::Eof => format!("expected end of file, but found {}", f),
        _ => format!("expected {}, found {}", token_name(expected), f),
    }
}
