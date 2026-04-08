use super::*;

impl Parser {
    pub(super) fn parse_top_decl(&mut self) -> Result<TopDecl, ParseError> {
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
        let type_params = self.parse_type_params()?;

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

        Ok(TopDecl::Scripture {
            name,
            type_params,
            fields,
        })
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
        let type_params = self.parse_type_params()?;
        self.expect(&Token::Indent)?;

        let mut variants = Vec::new();
        while !matches!(self.peek(), Token::Dedent | Token::Eof) {
            let variant_name = self.expect_ident()?;
            let fields = if self.peek() == &Token::Indent {
                self.advance();
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
            variants.push(CovenantVariantDecl {
                name: variant_name,
                fields,
            });
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

        Ok(TopDecl::Covenant {
            name,
            type_params,
            variants,
        })
    }

    fn parse_salm_decl(&mut self) -> Result<TopDecl, ParseError> {
        self.expect(&Token::Salm)?;
        let name = self.expect_ident()?;

        if self.peek() == &Token::Upon {
            self.advance();
            let target_type = self.expect_ident()?;
            let type_params = self.parse_type_params()?;
            let params = self.parse_optional_params()?;
            self.expect(&Token::Reveals)?;
            let ret_type = self.parse_type()?;
            let body = self.parse_block()?;
            Ok(TopDecl::MethodSalm {
                name,
                type_params,
                target_type,
                params,
                ret_type,
                body,
            })
        } else {
            let type_params = self.parse_type_params()?;
            let params = self.parse_optional_params()?;
            self.expect(&Token::Reveals)?;
            let ret_type = self.parse_type()?;
            let body = self.parse_block()?;
            Ok(TopDecl::Salm {
                name,
                type_params,
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
        loop {
            match self.peek() {
                Token::Comma => {
                    self.advance();
                    params.push(self.parse_param()?);
                }
                Token::And => {
                    self.advance();
                    params.push(self.parse_param()?);
                    break;
                }
                _ => break,
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

    pub(super) fn parse_type(&mut self) -> Result<HolyType, ParseError> {
        let sp = self.sp().clone();
        match self.advance().token {
            Token::Atom => Ok(HolyType::Atom),
            Token::Fractional => Ok(HolyType::Fractional),
            Token::Word => Ok(HolyType::Word),
            Token::Dogma => Ok(HolyType::Dogma),
            Token::Void => Ok(HolyType::Void),
            Token::Ident(n) => {
                if self.peek() == &Token::Of && self.is_type_start_ahead(1) {
                    self.advance();
                    let first = self.parse_type()?;
                    let mut type_args = vec![first];
                    if self.peek() == &Token::Thus {
                        self.advance();
                    } else {
                        loop {
                            match self.peek() {
                                Token::Comma if self.is_type_start_ahead(1) => {
                                    self.advance();
                                    type_args.push(self.parse_type()?);
                                    if self.peek() == &Token::Thus {
                                        self.advance();
                                        break;
                                    }
                                }
                                Token::And if self.is_type_start_ahead(1) => {
                                    self.advance();
                                    type_args.push(self.parse_type()?);
                                    break;
                                }
                                _ => break,
                            }
                        }
                    }
                    Ok(HolyType::Generic(n, type_args))
                } else {
                    Ok(HolyType::Custom(n))
                }
            }
            t => Err(ParseError::at(
                format!(
                    "invalid type {} — use: atom, fractional, word, dogma, void, grace, verdict or a type name",
                    token_name(&t)
                ),
                sp.line,
                sp.col,
            )),
        }
    }

    pub(super) fn is_type_start_ahead(&self, offset: usize) -> bool {
        let tok = self.tokens.get(self.pos + offset).map(|s| &s.token);
        matches!(
            tok,
            Some(Token::Atom)
                | Some(Token::Fractional)
                | Some(Token::Word)
                | Some(Token::Dogma)
                | Some(Token::Void)
                | Some(Token::Ident(_))
        )
    }

    fn parse_type_params(&mut self) -> Result<Vec<String>, ParseError> {
        if self.peek() != &Token::Of || !self.is_type_start_ahead(1) {
            return Ok(Vec::new());
        }
        self.advance();
        self.parse_ident_list()
    }

    pub(super) fn parse_call_type_args(&mut self) -> Result<Vec<HolyType>, ParseError> {
        if self.peek() != &Token::Of || !self.is_type_start_ahead(1) {
            return Ok(Vec::new());
        }
        self.advance();
        self.parse_type_list()
    }

    pub(super) fn parse_variant_type_args(&mut self) -> Result<Vec<HolyType>, ParseError> {
        if self.peek() == &Token::Of && self.is_type_start_ahead(1) {
            self.advance();
            self.parse_type_list()
        } else {
            Ok(Vec::new())
        }
    }

    pub(super) fn parse_ident_list(&mut self) -> Result<Vec<String>, ParseError> {
        let mut items = vec![self.expect_ident()?];
        loop {
            match self.peek() {
                Token::Comma => {
                    self.advance();
                    items.push(self.expect_ident()?);
                }
                Token::And => {
                    self.advance();
                    items.push(self.expect_ident()?);
                    break;
                }
                _ => break,
            }
        }
        Ok(items)
    }

    fn parse_type_list(&mut self) -> Result<Vec<HolyType>, ParseError> {
        let mut items = vec![self.parse_type()?];
        loop {
            match self.peek() {
                Token::Comma if self.is_type_start_ahead(1) => {
                    self.advance();
                    items.push(self.parse_type()?);
                }
                Token::And if self.is_type_start_ahead(1) => {
                    self.advance();
                    items.push(self.parse_type()?);
                    break;
                }
                _ => break,
            }
        }
        Ok(items)
    }

    pub(super) fn parse_builtin_covenant_name(&mut self) -> Result<String, ParseError> {
        self.expect_ident()
    }
}
