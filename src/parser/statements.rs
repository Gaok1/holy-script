use super::*;

impl Parser {
    pub(super) fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
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

    pub(super) fn parse_stmt(&mut self) -> Result<Stmt, ParseError> {
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
        self.advance();
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
                let args = self.parse_arg_list()?;
                if self.peek() == &Token::Thus {
                    self.advance();
                }
                args
            } else {
                Vec::new()
            };
            Ok(Stmt::MethodCallStmt {
                method: name,
                target,
                args,
            })
        } else {
            let type_args = self.parse_call_type_args()?;
            let args = if self.peek() == &Token::Praying {
                self.advance();
                let args = self.parse_arg_list()?;
                if self.peek() == &Token::Thus {
                    self.advance();
                }
                args
            } else {
                Vec::new()
            };
            Ok(Stmt::FnCallStmt {
                name,
                type_args,
                args,
            })
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
        let target = self.parse_expr()?;
        self.expect(&Token::Indent)?;

        let mut branches = Vec::new();
        while self.peek() == &Token::As {
            self.advance();
            let variant = self.expect_variant_name()?;
            let bindings = if self.peek() == &Token::Bearing {
                self.advance();
                self.parse_ident_list()?
            } else {
                Vec::new()
            };
            let body = self.parse_block()?;
            branches.push(DiscernBranch {
                variant,
                bindings,
                body,
            });
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
}
