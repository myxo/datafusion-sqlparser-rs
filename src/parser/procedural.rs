use super::*;

impl<'a> Parser<'a> {
    pub(super) fn parse_do_statement(&mut self) -> Result<DoStatement, ParserError> {
        let language_before = if self.parse_keyword(Keyword::LANGUAGE) {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        let body = self.parse_value()?;
        if !matches!(
            body.value,
            Value::SingleQuotedString(_)
                | Value::EscapedStringLiteral(_)
                | Value::UnicodeStringLiteral(_)
                | Value::DollarQuotedString(_)
        ) {
            return self.expected_ref("a string literal", self.get_current_token());
        }
        let language_after = if self.parse_keyword(Keyword::LANGUAGE) {
            Some(self.parse_identifier()?)
        } else {
            None
        };
        if language_before.is_some() && language_after.is_some() {
            return Err(ParserError::ParserError(
                "LANGUAGE specified more than once".into(),
            ));
        }
        Ok(DoStatement {
            body,
            language: language_before.clone().or(language_after),
            language_before_body: language_before.is_some(),
        })
    }

    /// Parse one complete PostgreSQL PL/pgSQL block.
    pub fn parse_plpgsql(&mut self) -> Result<PlPgSqlBlock, ParserError> {
        let declarations = if self.parse_keyword(Keyword::DECLARE) {
            let mut declarations = Vec::new();
            while !self.peek_keyword(Keyword::BEGIN) {
                if matches!(self.peek_token_ref().token, Token::EOF) {
                    return self.expected_ref("BEGIN", self.peek_token_ref());
                }
                let name = self.parse_identifier()?;
                let data_type = self.parse_data_type()?;
                let initializer = if self.consume_token(&Token::Assignment)
                    || self.consume_token(&Token::Eq)
                    || self.parse_keyword(Keyword::DEFAULT)
                {
                    Some(self.parse_expr()?)
                } else {
                    None
                };
                self.expect_token(&Token::SemiColon)?;
                declarations.push(PlPgSqlDeclaration {
                    name,
                    data_type,
                    initializer,
                });
            }
            declarations
        } else {
            Vec::new()
        };
        self.expect_keyword_is(Keyword::BEGIN)?;
        let statements = self.parse_plpgsql_statements(&[Keyword::END])?;
        self.expect_keyword_is(Keyword::END)?;
        let _ = self.consume_token(&Token::SemiColon);
        self.expect_token(&Token::EOF)?;
        Ok(PlPgSqlBlock {
            declarations,
            statements,
        })
    }

    fn parse_plpgsql_statements(
        &mut self,
        terminal_keywords: &[Keyword],
    ) -> Result<Vec<PlPgSqlStatement>, ParserError> {
        let _guard = self.recursion_counter.try_decrease()?;
        let mut statements = Vec::new();
        while !terminal_keywords
            .iter()
            .any(|keyword| self.peek_keyword(*keyword))
        {
            if matches!(self.peek_token_ref().token, Token::EOF) {
                return self.expected_ref("a PL/pgSQL statement", self.peek_token_ref());
            }
            statements.push(self.parse_plpgsql_statement()?);
            self.expect_token(&Token::SemiColon)?;
        }
        Ok(statements)
    }

    fn parse_plpgsql_statement(&mut self) -> Result<PlPgSqlStatement, ParserError> {
        if self.parse_keyword(Keyword::IF) {
            return self.parse_plpgsql_if();
        }
        if self.parse_keyword(Keyword::RETURN) {
            return self.parse_expr().map(PlPgSqlStatement::Return);
        }
        if self.parse_keyword(Keyword::GET) {
            self.expect_keyword_is(Keyword::DIAGNOSTICS)?;
            let target = self.parse_identifier()?;
            self.expect_token(&Token::Eq)?;
            self.expect_keyword_is(Keyword::ROW_COUNT)?;
            return Ok(PlPgSqlStatement::GetRowCount { target });
        }
        if self.parse_keyword(Keyword::RAISE) {
            self.expect_keyword_is(Keyword::EXCEPTION)?;
            let format = self.parse_value()?;
            if !matches!(
                format.value,
                Value::SingleQuotedString(_)
                    | Value::EscapedStringLiteral(_)
                    | Value::UnicodeStringLiteral(_)
                    | Value::DollarQuotedString(_)
            ) {
                return self.expected_ref("a string literal", self.get_current_token());
            }
            let mut arguments = Vec::new();
            while self.consume_token(&Token::Comma) {
                arguments.push(self.parse_expr()?);
            }
            let hint = if self.parse_keywords(&[Keyword::USING, Keyword::HINT]) {
                self.expect_token(&Token::Eq)?;
                Some(self.parse_expr()?)
            } else {
                None
            };
            return Ok(PlPgSqlStatement::RaiseException {
                format,
                arguments,
                hint,
            });
        }
        if matches!(
            &self.peek_token_ref().token,
            Token::Word(word)
                if matches!(
                    word.keyword,
                    Keyword::SELECT
                        | Keyword::WITH
                        | Keyword::VALUES
                        | Keyword::INSERT
                        | Keyword::UPDATE
                        | Keyword::DELETE
                )
        ) {
            return self
                .parse_statement()
                .map(Box::new)
                .map(PlPgSqlStatement::Sql);
        }

        let target = self.parse_object_name(false)?;
        if !self.consume_token(&Token::Assignment) {
            self.expect_token(&Token::Eq)?;
        }
        let value = self.parse_expr()?;
        Ok(PlPgSqlStatement::Assignment { target, value })
    }

    fn parse_plpgsql_if(&mut self) -> Result<PlPgSqlStatement, ParserError> {
        let mut branches = Vec::new();
        let condition = self.parse_expr()?;
        self.expect_keyword_is(Keyword::THEN)?;
        let statements = self.parse_plpgsql_statements(&[
            Keyword::ELSIF,
            Keyword::ELSEIF,
            Keyword::ELSE,
            Keyword::END,
        ])?;
        branches.push(PlPgSqlIfBranch {
            condition,
            statements,
        });
        while self.parse_keyword(Keyword::ELSIF) || self.parse_keyword(Keyword::ELSEIF) {
            let condition = self.parse_expr()?;
            self.expect_keyword_is(Keyword::THEN)?;
            let statements = self.parse_plpgsql_statements(&[
                Keyword::ELSIF,
                Keyword::ELSEIF,
                Keyword::ELSE,
                Keyword::END,
            ])?;
            branches.push(PlPgSqlIfBranch {
                condition,
                statements,
            });
        }
        let else_statements = if self.parse_keyword(Keyword::ELSE) {
            Some(self.parse_plpgsql_statements(&[Keyword::END])?)
        } else {
            None
        };
        self.expect_keywords(&[Keyword::END, Keyword::IF])?;
        Ok(PlPgSqlStatement::If {
            branches,
            else_statements,
        })
    }
}
