// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::ast::{Expression, Identifier, LetStatement, Program, Statement};
use crate::lexer::Lexer;
use crate::token::{Token, TokenType};

pub struct Parser<'a> {
    lexer: &'a mut Lexer,
    current_token: Option<Token>,
    peek_token: Option<Token>,
    errors: Vec<String>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: None,
            peek_token: None,
            errors: Vec::new(),
        };

        parser.next_token();
        parser.next_token();

        parser
    }

    pub fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    fn peek_error(&mut self, token_type: &TokenType) {
        let got = self
            .peek_token
            .as_ref()
            .map_or(&TokenType::Eof, |token| &token.token_type);

        let message = format!("expected next token to be {token_type}, got {got} instead",);
        self.errors.push(message);
    }

    pub fn next_token(&mut self) {
        self.current_token = self.peek_token.replace(self.lexer.next_token());
    }

    pub fn parse_program(&mut self) -> Program {
        let mut program = Program {
            statements: Vec::new(),
        };

        while self.current_token.is_some() {
            if self.cur_token_is(&TokenType::Eof) {
                break;
            }
            if let Some(stmt) = self.parse_statement() {
                program.statements.push(stmt);
            }
            self.next_token();
        }

        program
    }

    fn parse_statement(&mut self) -> Option<Statement> {
        match &self.current_token {
            Some(token) => match token.token_type {
                TokenType::Let => self.parse_let_statement(),
                _ => None,
            },
            None => None,
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        let token = self.current_token.clone()?;

        if !self.cur_token_is(&TokenType::Let) {
            return None;
        }

        if !self.expect_peek(&TokenType::Ident) {
            return None;
        }

        let name_token = self.current_token.clone()?;
        let name = Identifier {
            token: name_token.clone(),
            value: name_token.literal.clone(),
        };

        if !self.expect_peek(&TokenType::Assign) {
            return None;
        }

        while self.current_token.is_some() {
            if !self.cur_token_is(&TokenType::SemiColon) {
                break;
            }
            self.next_token();
        }

        Some(Statement::Let(LetStatement {
            token,
            name,
            value: Expression::Identifier(Identifier {
                token: name_token.clone(),
                value: name_token.literal,
            }),
        }))
    }

    fn cur_token_is(&self, token_type: &TokenType) -> bool {
        self.current_token
            .as_ref()
            .is_some_and(|token| token.token_type == *token_type)
    }

    fn peek_token_is(&self, token_type: &TokenType) -> bool {
        self.peek_token
            .as_ref()
            .is_some_and(|token| token.token_type == *token_type)
    }

    fn expect_peek(&mut self, token_type: &TokenType) -> bool {
        if self.peek_token_is(token_type) {
            self.next_token();
            true
        } else {
            self.peek_error(token_type);
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Node, Statement};

    #[test]
    fn test_let_statements() {
        let input = r"let x = 5;
let y = 10;
let foobar = 838383;
";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = parser.parse_program();
        check_parser_errors(&parser);
        assert_eq!(program.statements.len(), 3);

        let tests = ["x", "y", "foobar"];

        for (i, expected_identifier) in tests.iter().enumerate() {
            let statement = &program.statements[i];
            if !test_let_statement(statement, expected_identifier) {
                return;
            }
        }
    }

    fn test_let_statement(statement: &Statement, name: &str) -> bool {
        if statement.token_literal() != "let" {
            eprintln!("token_literal not 'let. got={}", statement.token_literal());
            return false;
        }

        match statement {
            Statement::Let(let_stmt) => {
                if let_stmt.name.value != name {
                    eprintln!(
                        "let_stmt.name.value not '{}. got={}",
                        name, let_stmt.name.value
                    );
                    return false;
                }

                if let_stmt.name.token_literal() != name {
                    eprintln!("let_stmt.name not '{:#?}. got={:#?}", name, let_stmt.name);
                    return false;
                }
                true
            }
        }
    }

    fn check_parser_errors(parser: &Parser) {
        let errors = parser.errors();

        if errors.len() == 0 {
            return;
        }

        eprintln!("parser has {} errors", errors.len());
        for e in &errors {
            eprintln!("parser error: {e}");
        }

        panic!()
    }
}
