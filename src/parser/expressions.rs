use super::*;

impl Parser {
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
                    let r: Expr = self.parse_unary()?;
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
            Token::After => {
                self.advance();
                let inner = self.parse_expr()?;
                let thus_sp = self.sp().clone();
                if self.peek() != &Token::Thus {
                    return Err(ParseError::at(
                        format!(
                            "expected 'thus' to close 'after' grouping, found {}",
                            token_name(self.peek())
                        ),
                        thus_sp.line,
                        thus_sp.col,
                    ));
                }
                self.advance();
                Ok(inner)
            }
            Token::Hail => {
                self.advance();
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
                    Ok(Expr::MethodCall {
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
                    Ok(Expr::FnCall {
                        name,
                        type_args,
                        args,
                    })
                }
            }
            Token::Manifest => {
                self.advance();
                let name = self.expect_variant_name()?;
                if self.peek() == &Token::Of {
                    let next = self.tokens.get(self.pos + 1).map(|s| &s.token);
                    let is_covenant = matches!(next, Some(Token::Ident(_)));
                    if is_covenant {
                        self.advance();
                        let covenant = self.parse_builtin_covenant_name()?;
                        let type_args = self.parse_variant_type_args()?;
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
                        return Ok(Expr::ManifestVariant {
                            variant: name,
                            covenant,
                            type_args,
                            args,
                        });
                    }
                }
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
                Ok(Expr::Manifest {
                    scripture: name,
                    args,
                })
            }
            Token::Ident(name) => {
                self.advance();
                if self.peek() == &Token::Of {
                    let next = self.tokens.get(self.pos + 1).map(|s| &s.token);
                    let is_covenant = matches!(next, Some(Token::Ident(_)));
                    if is_covenant {
                        self.advance();
                        let covenant = self.parse_builtin_covenant_name()?;
                        let type_args = self.parse_variant_type_args()?;
                        return Ok(Expr::TypedUnitVariant {
                            variant: name,
                            covenant,
                            type_args,
                        });
                    }
                }
                if self.peek() == &Token::From {
                    self.advance();
                    self.parse_from_target(name)
                } else {
                    Ok(Expr::Var(name))
                }
            }
            Token::IntLit(n) => {
                self.advance();
                Ok(Expr::Lit(Literal::Int(n)))
            }
            Token::FloatLit(f) => {
                self.advance();
                Ok(Expr::Lit(Literal::Float(f)))
            }
            Token::StrLit(s) => {
                self.advance();
                Ok(Expr::Lit(Literal::Str(s)))
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

    fn parse_from_target(&mut self, field: String) -> Result<Expr, ParseError> {
        if let Token::Its = self.peek() {
            self.advance();
            return Ok(Expr::SelfFieldAccess { field });
        }

        let object = self.parse_atom()?;
        Ok(Expr::FieldAccess {
            field,
            object: Box::new(object),
        })
    }

    pub(super) fn parse_arg_list(&mut self) -> Result<Vec<Expr>, ParseError> {
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
