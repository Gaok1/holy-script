use crate::ast::*;
use crate::lexer::{Spanned, Token, token_name};

// ══════════════════════════════════════════════════════════════════
// Parse error with source position
// ══════════════════════════════════════════════════════════════════

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
        write!(
            f,
            "line {}, column {}: {}",
            self.line, self.col, self.message
        )
    }
}

// ══════════════════════════════════════════════════════════════════
// Parser
// ══════════════════════════════════════════════════════════════════

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

    /// Consumes the expected token or returns a contextual error message.
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

    // ──────────────────────────────────────────────────────────────
    // Program
    // ──────────────────────────────────────────────────────────────

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
            let mut items = vec![self.expect_ident()?];
            while self.peek() == &Token::Comma {
                self.advance();
                items.push(self.expect_ident()?);
            }
            Some(items)
        } else {
            None
        };
        Ok(Testament { name, revealing })
    }

    // ──────────────────────────────────────────────────────────────
    // Top-level declarations
    // ──────────────────────────────────────────────────────────────

    fn parse_top_decl(&mut self) -> Result<TopDecl, ParseError> {
        match self.peek().clone() {
            Token::Scripture => self.parse_scripture(),
            Token::Sin => self.parse_sin_decl(),
            Token::Covenant => self.parse_covenant_decl(),
            Token::Salm => self.parse_salm_decl(),
            t => {
                let sp = self.sp().clone();
                Err(ParseError::at(
                    format!(
                        "{} cannot start a declaration — use 'salm', 'scripture', 'sin' or 'covenant'",
                        token_name(&t)
                    ),
                    sp.line,
                    sp.col,
                ))
            }
        }
    }

    fn parse_scripture(&mut self) -> Result<TopDecl, ParseError> {
        self.expect(&Token::Scripture)?;
        let name = self.expect_ident()?;

        let mut fields = Vec::new();

        self.expect(&Token::Indent)?;

        while !matches!(self.peek(), Token::Dedent | Token::Eof) {
            let fname = self.expect_ident()?;
            self.expect(&Token::Of)?;
            let ty = self.parse_type()?;
            fields.push((fname, ty));
        }
        self.expect(&Token::Dedent)?;
        if fields.is_empty() {
            let sp = self.sp().clone();
            return Err(ParseError::at(
                format!("scripture '{}' must have at least one field", name),
                sp.line,
                sp.col,
            ));
        }
        Ok(TopDecl::Scripture { name, fields })
    }

    fn parse_sin_decl(&mut self) -> Result<TopDecl, ParseError> {
        self.expect(&Token::Sin)?;
        let name = self.expect_ident()?;

        let mut fields = Vec::new();

        if self.peek() == &Token::Indent {
            self.advance();
            while !matches!(self.peek(), Token::Dedent | Token::Eof) {
                let fname = self.expect_ident()?;
                self.expect(&Token::Of)?;
                let ty = self.parse_type()?;
                fields.push((fname, ty));
            }
            self.expect(&Token::Dedent)?;
        }
        Ok(TopDecl::SinDecl { name, fields })
    }

    fn parse_covenant_decl(&mut self) -> Result<TopDecl, ParseError> {
        self.expect(&Token::Covenant)?;
        let name = self.expect_ident()?;
        self.expect(&Token::Indent)?;

        let mut variants = Vec::new();
        while !matches!(self.peek(), Token::Dedent | Token::Eof) {
            let variant_name = self.expect_ident()?;
            let fields = if self.peek() == &Token::Indent {
                self.advance(); // consume Indent
                let mut fs = Vec::new();
                while !matches!(self.peek(), Token::Dedent | Token::Eof) {
                    let fname = self.expect_ident()?;
                    self.expect(&Token::Of)?;
                    let ty = self.parse_type()?;
                    fs.push((fname, ty));
                }
                self.expect(&Token::Dedent)?;
                fs
            } else {
                Vec::new()
            };
            variants.push(CovenantVariantDecl { name: variant_name, fields });
        }

        self.expect(&Token::Dedent)?;
        if variants.is_empty() {
            let sp = self.sp().clone();
            return Err(ParseError::at(
                format!("covenant '{}' must have at least one variant", name),
                sp.line,
                sp.col,
            ));
        }

        Ok(TopDecl::Covenant { name, variants })
    }

    fn parse_salm_decl(&mut self) -> Result<TopDecl, ParseError> {
        self.expect(&Token::Salm)?;
        let name = self.expect_ident()?;

        if self.peek() == &Token::Upon {
            self.advance();
            let target_type = self.expect_ident()?;
            let params = self.parse_optional_params()?;
            self.expect(&Token::Reveals)?;
            let ret_type = self.parse_type()?;
            let body = self.parse_block()?;
            Ok(TopDecl::MethodSalm {
                name,
                target_type,
                params,
                ret_type,
                body,
            })
        } else {
            let params = self.parse_optional_params()?;
            self.expect(&Token::Reveals)?;
            let ret_type = self.parse_type()?;
            let body = self.parse_block()?;
            Ok(TopDecl::Salm {
                name,
                params,
                ret_type,
                body,
            })
        }
    }

    fn parse_optional_params(&mut self) -> Result<Vec<(String, HolyType)>, ParseError> {
        if self.peek() == &Token::Receiving {
            self.advance();
            self.parse_param_list()
        } else {
            Ok(Vec::new())
        }
    }

    fn parse_param_list(&mut self) -> Result<Vec<(String, HolyType)>, ParseError> {
        let mut params = vec![self.parse_param()?];
        while self.peek() == &Token::Comma {
            self.advance();
            params.push(self.parse_param()?);
            
        }
        loop {
            match self.peek() {
                &Token::Comma => {
                    self.advance();
                    params.push(self.parse_param()?); continue;
                }
                &Token::And => {
                     self.advance();
                     params.push(self.parse_param()?);
                     break;
                }
                _ => break
            }
        }

        Ok(params)
    }

    fn parse_param(&mut self) -> Result<(String, HolyType), ParseError> {
        let name = self.expect_ident()?;
        self.expect(&Token::Of)?;
        let ty = self.parse_type()?;
        Ok((name, ty))
    }

    // ──────────────────────────────────────────────────────────────
    // Types
    // ──────────────────────────────────────────────────────────────

    fn parse_type(&mut self) -> Result<HolyType, ParseError> {
        let sp = self.sp().clone();
        match self.advance().token {
            Token::Atom => Ok(HolyType::Atom),
            Token::Fractional => Ok(HolyType::Fractional),
            Token::Word => Ok(HolyType::Word),
            Token::Dogma => Ok(HolyType::Dogma),
            Token::Void => Ok(HolyType::Void),
            Token::Ident(n) => Ok(HolyType::Custom(n)),

            t => Err(ParseError::at(
                format!(
                    "invalid type {} — use: atom, fractional, word, dogma, void or a scripture name",
                    token_name(&t)
                ),
                sp.line,
                sp.col,
            )),
        }
    }

    // ──────────────────────────────────────────────────────────────
    // Block
    // ──────────────────────────────────────────────────────────────

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let sp = self.sp().clone();
        self.expect(&Token::Indent)?;
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Token::Dedent | Token::Eof) {
            stmts.push(self.parse_stmt()?);
        }
        self.expect(&Token::Dedent)?;
        if stmts.is_empty() {
            return Err(ParseError::at(
                "empty block — add at least one statement",
                sp.line,
                sp.col,
            ));
        }
        Ok(stmts)
    }

    // ──────────────────────────────────────────────────────────────
    // Statements
    // ──────────────────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
        let sp = self.sp().clone();
        match self.peek().clone() {
            Token::Let => self.parse_decl(),
            Token::Hail => self.parse_invocation_stmt(),
            Token::Reveal => self.parse_reveal_stmt(),
            Token::Whether => self.parse_conditional(),
            Token::Litany => self.parse_litany(),
            Token::Confess => self.parse_sin_handler(),
            Token::Discern => self.parse_discern(),
            Token::Transgress => self.parse_transgress(),
            Token::Forsake => {
                self.advance();
                Ok(Stmt::Forsake)
            }
            Token::Ascend => {
                self.advance();
                Ok(Stmt::Ascend)
            }
            Token::Ident(_) => self.parse_assign(),
            t => Err(ParseError::at(
                format!(
                    "{} cannot start a statement — use 'let there', 'hail', 'whether', 'litany for', 'confess', 'discern', 'transgress', 'reveal', 'forsake', 'ascend' or a variable followed by 'become'",
                    token_name(&t)
                ),
                sp.line,
                sp.col,
            )),
        }
    }

    fn parse_decl(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Let)?;
        self.expect(&Token::There)?;

        if self.peek() == &Token::Be {
            self.advance();
            let name = self.expect_ident()?;
            self.expect(&Token::Of)?;
            let ty = self.parse_type()?;
            Ok(Stmt::DeclNoVal { name, ty })
        } else {
            let name = self.expect_ident()?;
            self.expect(&Token::Of)?;
            let ty = self.parse_type()?;
            self.expect(&Token::Be)?;
            let val = self.parse_expr()?;
            Ok(Stmt::DeclVal { name, ty, val })
        }
    }

    fn parse_assign(&mut self) -> Result<Stmt, ParseError> {
        let sp = self.sp().clone();
        let name = self.expect_ident()?;
        if self.peek() != &Token::Become {
            let found = self.peek().clone();
            return Err(ParseError::at(
                format!(
                    "expected 'become' to reassign '{}', found {} — to call a function use 'hail'",
                    name,
                    token_name(&found)
                ),
                sp.line,
                sp.col,
            ));
        }
        self.advance(); // consume 'become'
        let val = self.parse_expr()?;
        Ok(Stmt::Assign { name, val })
    }

    fn parse_invocation_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Hail)?;
        let name = self.expect_ident()?;
        if self.peek() == &Token::Upon {
            self.advance();
            let target = self.expect_ident()?;
            let args = if self.peek() == &Token::Praying {
                self.advance();
                self.parse_arg_list()?
            } else {
                Vec::new()
            };
            Ok(Stmt::MethodCallStmt {
                method: name,
                target,
                args,
            })
        } else {
            let args = if self.peek() == &Token::Praying {
                self.advance();
                self.parse_arg_list()?
            } else {
                Vec::new()
            };
            Ok(Stmt::FnCallStmt { name, args })
        }
    }

    fn parse_reveal_stmt(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Reveal)?;
        Ok(Stmt::Reveal(self.parse_expr()?))
    }

    fn parse_conditional(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Whether)?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        let mut branches = vec![(cond, body)];
        let mut otherwise = None;

        loop {
            if self.peek() == &Token::Otherwise {
                self.advance();
                if self.peek() == &Token::So {
                    self.advance();
                    let cond = self.parse_expr()?;
                    let body = self.parse_block()?;
                    branches.push((cond, body));
                } else {
                    otherwise = Some(self.parse_block()?);
                    break;
                }
            } else {
                break;
            }
        }

        Ok(Stmt::Conditional {
            branches,
            otherwise,
        })
    }

    fn parse_litany(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Litany)?;
        self.expect(&Token::For)?;
        let cond = self.parse_expr()?;
        let body = self.parse_block()?;
        Ok(Stmt::Litany { cond, body })
    }

    fn parse_sin_handler(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Confess)?;
        let try_block = self.parse_block()?;

        let mut handlers = Vec::new();
        while self.peek() == &Token::Answer {
            self.advance();
            self.expect(&Token::For)?;
            let sin_type = self.expect_ident()?;
            let binding = if self.peek() == &Token::As {
                self.advance();
                Some(self.expect_ident()?)
            } else {
                None
            };
            let body = self.parse_block()?;
            handlers.push(SinHandler {
                sin_type,
                binding,
                body,
            });
        }

        if handlers.is_empty() {
            let sp = self.sp().clone();
            return Err(ParseError::at(
                "'confess' block requires at least one 'answer for <SinType>'",
                sp.line,
                sp.col,
            ));
        }

        let absolve = if self.peek() == &Token::Absolve {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::Confess {
            try_block,
            handlers,
            absolve,
        })
    }

    fn parse_transgress(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Transgress)?;
        let sin_type = self.expect_ident()?;
        let args = if self.peek() == &Token::Praying {
            self.advance();
            self.parse_arg_list()?
        } else {
            Vec::new()
        };
        Ok(Stmt::Transgress { sin_type, args })
    }

    fn parse_discern(&mut self) -> Result<Stmt, ParseError> {
        self.expect(&Token::Discern)?;
        let target = self.expect_ident()?;
        self.expect(&Token::Indent)?;

        let mut branches = Vec::new();
        while self.peek() == &Token::As {
            self.advance();
            let variant = self.expect_ident()?;
            let bindings = if self.peek() == &Token::Bearing {
                self.advance();
                let mut bs = vec![self.expect_ident()?];
                while self.peek() == &Token::Comma {
                    self.advance();
                    bs.push(self.expect_ident()?);
                }
                bs
            } else {
                Vec::new()
            };
            let body = self.parse_block()?;
            branches.push(DiscernBranch { variant, bindings, body });
        }

        if branches.is_empty() {
            let sp = self.sp().clone();
            return Err(ParseError::at(
                "'discern' block requires at least one 'as <Variant>' clause",
                sp.line,
                sp.col,
            ));
        }

        let otherwise = if self.peek() == &Token::Otherwise {
            self.advance();
            Some(self.parse_block()?)
        } else {
            None
        };

        self.expect(&Token::Dedent)?;
        Ok(Stmt::Discern {
            target,
            branches,
            otherwise,
        })
    }

    // ──────────────────────────────────────────────────────────────
    // Expressions  (cmp > arith > term > unary > atom)
    // ──────────────────────────────────────────────────────────────

    pub fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_arith_expr()?;

        if let Some(op) = self.try_cmp_op()? {
            let right = self.parse_arith_expr()?;
            Ok(Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            })
        } else {
            Ok(left)
        }
    }

    fn try_cmp_op(&mut self) -> Result<Option<BinOp>, ParseError> {
        match self.peek().clone() {
            Token::Is => {
                self.advance();
                if self.peek() == &Token::Not {
                    self.advance();
                    Ok(Some(BinOp::Ne))
                } else {
                    Ok(Some(BinOp::Eq))
                }
            }
            Token::Greater => {
                self.advance();
                if self.peek() == &Token::Than {
                    self.advance();
                }
                Ok(Some(BinOp::Gt))
            }
            Token::Lesser => {
                self.advance();
                if self.peek() == &Token::Than {
                    self.advance();
                }
                Ok(Some(BinOp::Lt))
            }
            Token::No => {
                let sp = self.sp().clone();
                self.advance();
                match self.peek().clone() {
                    Token::Greater => {
                        self.advance();
                        if self.peek() == &Token::Than {
                            self.advance();
                        }
                        Ok(Some(BinOp::Le))
                    }
                    Token::Lesser => {
                        self.advance();
                        if self.peek() == &Token::Than {
                            self.advance();
                        }
                        Ok(Some(BinOp::Ge))
                    }
                    t => Err(ParseError::at(
                        format!(
                            "expected 'greater' or 'lesser' after 'no', found {}",
                            token_name(&t)
                        ),
                        sp.line,
                        sp.col,
                    )),
                }
            }
            _ => Ok(None),
        }
    }

    fn parse_arith_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_term()?;
        loop {
            match self.peek() {
                Token::Plus => {
                    self.advance();
                    let r = self.parse_term()?;
                    left = Expr::BinOp {
                        op: BinOp::Add,
                        left: Box::new(left),
                        right: Box::new(r),
                    };
                }
                Token::Minus => {
                    self.advance();
                    let r = self.parse_term()?;
                    left = Expr::BinOp {
                        op: BinOp::Sub,
                        left: Box::new(left),
                        right: Box::new(r),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_term(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            match self.peek() {
                Token::Times => {
                    self.advance();
                    let r = self.parse_unary()?;
                    left = Expr::BinOp {
                        op: BinOp::Mul,
                        left: Box::new(left),
                        right: Box::new(r),
                    };
                }
                Token::Over => {
                    self.advance();
                    let r = self.parse_unary()?;
                    left = Expr::BinOp {
                        op: BinOp::Div,
                        left: Box::new(left),
                        right: Box::new(r),
                    };
                }
                Token::Remainder => {
                    self.advance();
                    let r = self.parse_unary()?;
                    left = Expr::BinOp {
                        op: BinOp::Rem,
                        left: Box::new(left),
                        right: Box::new(r),
                    };
                }
                _ => break,
            }
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek() == &Token::Negate {
            self.advance();
            Ok(Expr::Negate(Box::new(self.parse_atom()?)))
        } else {
            self.parse_atom()
        }
    }

    fn parse_atom(&mut self) -> Result<Expr, ParseError> {
        let sp = self.sp().clone();
        match self.peek().clone() {
            Token::Hail => {
                self.advance();
                let name = self.expect_ident()?;
                if self.peek() == &Token::Upon {
                    self.advance();
                    let target = self.expect_ident()?;
                    let args = if self.peek() == &Token::Praying {
                        self.advance();
                        self.parse_arg_list()?
                    } else {
                        Vec::new()
                    };
                    Ok(Expr::MethodCall {
                        method: name,
                        target,
                        args,
                    })
                } else {
                    let args = if self.peek() == &Token::Praying {
                        self.advance();
                        self.parse_arg_list()?
                    } else {
                        Vec::new()
                    };
                    Ok(Expr::FnCall { name, args })
                }
            }
            Token::Manifest => {
                self.advance();
                let scripture = self.expect_ident()?;
                let args = if self.peek() == &Token::Praying {
                    self.advance();
                    self.parse_arg_list()?
                } else {
                    Vec::new()
                };
                Ok(Expr::Manifest { scripture, args })
            }
            Token::Ident(name) => {
                self.advance();
                if self.peek() == &Token::From {
                    self.advance();
                    self.parse_from_target(name)
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Token::IntLit(n) => {
                let v = n;
                self.advance();
                Ok(Expr::Lit(Literal::Int(v)))
            }
            Token::FloatLit(f) => {
                let v = f;
                self.advance();
                Ok(Expr::Lit(Literal::Float(v)))
            }
            Token::StrLit(s) => {
                let v = s.clone();
                self.advance();
                Ok(Expr::Lit(Literal::Str(v)))
            }
            Token::Blessed => {
                self.advance();
                Ok(Expr::Lit(Literal::Bool(true)))
            }
            Token::Forsaken => {
                self.advance();
                Ok(Expr::Lit(Literal::Bool(false)))
            }
            t => Err(ParseError::at(
                format!(
                    "{} is not a valid expression — expected: number, string, 'blessed', 'forsaken', variable, 'hail' or 'manifest'",
                    token_name(&t)
                ),
                sp.line,
                sp.col,
            )),
        }
    }

    /// Called after consuming `fieldName from` — parses the source of the access.
    /// Supports arbitrary nesting: `b from fieldComposite from its`
    fn parse_from_target(&mut self, field: String) -> Result<Expr, ParseError> {
        if let &Token::Its = self.peek() {
            self.advance();
            return Ok(Expr::SelfFieldAccess { field });
        }

        let object_name = self.expect_ident()?;
        if self.peek() == &Token::From {
            self.advance();
            let inner = self.parse_from_target(object_name)?;
            Ok(Expr::FieldAccess {
                field,
                object: Box::new(inner),
            })
        } else {
            Ok(Expr::FieldAccess {
                field,
                object: Box::new(Expr::Var(object_name)),
            })
        }
    }

    fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
        let mut args = vec![self.parse_expr()?];
        loop {
            match self.peek() {
                Token::Comma => {
                    self.advance();
                    args.push(self.parse_expr()?);
                }
                Token::And => {
                    self.advance();
                    args.push(self.parse_expr()?);
                    break;
                }
                _ => break,
            }
        }
        Ok(args)
    }
}

// ══════════════════════════════════════════════════════════════════
// Contextual expect messages
// ══════════════════════════════════════════════════════════════════

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
