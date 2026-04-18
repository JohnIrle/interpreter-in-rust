// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::ast::{
    Expression, ExpressionStatement, Identifier, IntegerLiteral, LetStatement, PrefixExpression,
    Program, ReturnStatement, Statement,
};
use crate::lexer::Lexer;
use crate::parser::Precedence::Lowest;
use crate::token::{Token, TokenType};
use std::collections::HashMap;

type PrefixParseFn<'a> = fn(&mut Parser<'a>) -> Option<Expression>;
type InfixParseFn = fn(Expression) -> Expression;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
enum Precedence {
    Lowest = 0,
    Equals,
    LessGreater,
    Sum,
    Product,
    Prefix,
    Call,
}

pub struct Parser<'a> {
    lexer: &'a mut Lexer,
    current_token: Option<Token>,
    peek_token: Option<Token>,
    errors: Vec<String>,

    prefix_parse_fns: HashMap<TokenType, PrefixParseFn<'a>>,
    infix_parse_fns: HashMap<TokenType, InfixParseFn>,
}

impl<'a> Parser<'a> {
    pub fn new(lexer: &'a mut Lexer) -> Self {
        let mut parser = Parser {
            lexer,
            current_token: None,
            peek_token: None,
            errors: Vec::new(),
            prefix_parse_fns: HashMap::new(),
            infix_parse_fns: HashMap::new(),
        };

        parser.register_prefix(TokenType::Ident, Parser::parse_identifier);
        parser.register_prefix(TokenType::Int, Parser::parse_integer_literal);
        parser.register_prefix(TokenType::Bang, Parser::parse_prefix_expression);
        parser.register_prefix(TokenType::Minus, Parser::parse_prefix_expression);

        parser.next_token();
        parser.next_token();

        parser
    }

    pub fn errors(&self) -> Vec<String> {
        self.errors.clone()
    }

    fn register_prefix(&mut self, token_type: TokenType, function: PrefixParseFn<'a>) {
        self.prefix_parse_fns.insert(token_type, function);
    }

    fn register_infix(&mut self, token_type: TokenType, function: InfixParseFn) {
        self.infix_parse_fns.insert(token_type, function);
    }

    fn peek_error(&mut self, token_type: &TokenType) {
        let got = self
            .peek_token
            .as_ref()
            .map_or(&TokenType::Eof, |token| &token.token_type);

        let message = format!("expected next token to be {token_type}, got {got} instead");
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
                TokenType::Return => self.parse_return_statement(),
                _ => self.parse_expression_statement(),
            },
            None => None,
        }
    }

    fn parse_let_statement(&mut self) -> Option<Statement> {
        let token = self.current_token.clone()?;

        if !self.expect_peek(&TokenType::Ident) {
            return None;
        }

        let name_token = self.current_token.clone()?;
        let name = Identifier {
            token: name_token.clone(),
            value: name_token.literal,
        };

        if !self.expect_peek(&TokenType::Assign) {
            return None;
        }

        self.next_token();

        let value = self.parse_expression(&Lowest);

        if self.peek_token_is(&TokenType::SemiColon) {
            self.next_token();
        }

        Some(Statement::Let(LetStatement {
            token,
            name: Expression::Identifier(name),
            value,
        }))
    }

    fn parse_return_statement(&mut self) -> Option<Statement> {
        let token = self.current_token.clone()?;

        self.next_token();

        let return_value = self.parse_expression(&Lowest);

        if self.peek_token_is(&TokenType::SemiColon) {
            self.next_token();
        }

        Some(Statement::Return(ReturnStatement {
            token,
            return_value,
        }))
    }

    fn parse_expression_statement(&mut self) -> Option<Statement> {
        let token = self.current_token.clone()?;
        let expression = self.parse_expression(&Precedence::Lowest);

        if self.peek_token_is(&TokenType::SemiColon) {
            self.next_token();
        }

        Some(Statement::Expression(ExpressionStatement {
            token,
            expression,
        }))
    }

    fn parse_integer_literal(&mut self) -> Option<Expression> {
        let token = self.current_token.clone()?;

        let Ok(value) = token.literal.parse::<i64>() else {
            self.errors
                .push(format!("could not parse {}", token.literal));
            return None;
        };

        Some(Expression::IntegerLiteral(IntegerLiteral { token, value }))
    }

    fn no_prefix_parse_fn_error(&mut self, token_type: &TokenType) {
        self.errors
            .push(format!("no prefix parse function for {token_type} found"));
    }

    fn parse_expression(&mut self, precedence: &Precedence) -> Option<Expression> {
        let token = self.current_token.clone()?;
        let prefix = self.prefix_parse_fns.get(&token.token_type);
        if let Some(prefix_fn) = prefix {
            let left_exp = prefix_fn(self);
            return left_exp;
        }
        self.no_prefix_parse_fn_error(&token.token_type);
        None
    }

    fn parse_identifier(&mut self) -> Option<Expression> {
        let token = self.current_token.clone()?;
        Some(Expression::Identifier(Identifier {
            token: token.clone(),
            value: token.literal,
        }))
    }

    fn parse_prefix_expression(&mut self) -> Option<Expression> {
        let token = self.current_token.clone()?;
        let operator = token.literal.clone();

        self.next_token();

        let right = self.parse_expression(&Precedence::Prefix)?;

        Some(Expression::Prefix(Box::from(PrefixExpression {
            token,
            operator,
            right: Box::new(right),
        })))
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
        let tests = [
            ("let x = 5;", "x", 5),
            ("let y = 10;", "y", 10),
            ("let foobar = 8383;", "foobar", 8383),
        ];

        for (i, test_case) in tests.iter().enumerate() {
            let mut lexer = Lexer::new(test_case.0);
            let mut parser = Parser::new(&mut lexer);

            let program = parser.parse_program();
            check_parser_errors(&parser);
            assert_eq!(program.statements.len(), 1);

            let statement = &program.statements[i];
            if !test_let_statement(statement, test_case.0) {
                return;
            }
        }
    }

    #[test]
    fn test_return_statements() {
        let input = r"return 5;
return 10;
return 993322;";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = &parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 3);

        for statement in &program.statements {
            if let Statement::Return(return_stmt) = statement {
                assert_eq!(
                    return_stmt.token_literal(),
                    "return",
                    "returnStmt.TokenLiteral not 'return' got {}",
                    return_stmt.token_literal()
                );
            } else {
                panic!("stmt ot ReturnStatement");
            }
        }
    }

    #[test]
    fn test_identifier_expression() {
        let input = "foobar;";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();

        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        if let Statement::Expression(statement) = &program.statements[0] {
            if let Some(expression) = &statement.expression {
                match expression {
                    Expression::Identifier(ident) => {
                        let value = &ident.value;
                        assert_eq!(value, "foobar", "ident value not foobar, got {value}");

                        let token_literal = ident.token_literal();
                        assert_eq!(
                            token_literal, "foobar",
                            "ident token_literal not foobar, got {token_literal}"
                        );
                    }
                    _ => panic!("Expression is not Identifier"),
                }
            } else {
                panic!("expression is not some");
            }
        } else {
            panic!("program.statements[0] is not ExpressionStatement");
        }
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = (parser).parse_program();

        check_parser_errors(&parser);

        dbg!(&program.statements);

        assert_eq!(program.statements.len(), 1);

        if let Statement::Expression(statement) = &program.statements[0] {
            if let Some(expression) = &statement.expression {
                match expression {
                    Expression::IntegerLiteral(integer_literal) => {
                        let value = &integer_literal.value;
                        assert_eq!(*value, 5, "literal value  not 5 got {value}");

                        let token_literal = integer_literal.token_literal();
                        assert_eq!(
                            token_literal, "5",
                            "token literal is not '5' got {token_literal}"
                        );
                    }
                    _ => panic!("Expression is not Integer literal"),
                }
            } else {
                panic!("expression is not some");
            }
        } else {
            panic!("program.statements[0] is not ExpressionStatement");
        }
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        let tests = [("!5;", "!", 5), ("-15;", "-", 15)];

        for (i, test_case) in tests.iter().enumerate() {
            let mut lexer = Lexer::new(test_case.0);
            let mut parser = Parser::new(&mut lexer);

            let program = parser.parse_program();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            if let Statement::Expression(statement) = &program.statements[0] {
                if let Some(expression) = &statement.expression {
                    match expression {
                        Expression::Prefix(expression) => {
                            let operator = &expression.operator;
                            assert_eq!(operator, test_case.1);

                            assert!(test_integer_literal(&expression.right, test_case.2));
                        }
                        _ => panic!("Expression is not prefix expression"),
                    }
                } else {
                    panic!("expression is not some");
                }
            } else {
                panic!("program.statements[0] is not ExpressionStatement");
            }
        }
    }

    fn test_let_statement(statement: &Statement, name: &str) -> bool {
        if statement.token_literal() != "let" {
            eprintln!("token_literal not 'let. got={}", statement.token_literal());
            return false;
        }

        match statement {
            Statement::Let(let_stmt) => match &let_stmt.name {
                Expression::Identifier(ident) => {
                    if ident.value != name {
                        eprintln!("let_stmt.name.value not '{}. got={}", name, ident.value);
                        return false;
                    }

                    if let_stmt.name.token_literal() != name {
                        eprintln!("let_stmt.name not '{:#?}. got={:#?}", name, let_stmt.name);
                        return false;
                    }
                    true
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn test_integer_literal(integer_literal: &Expression, value: i64) -> bool {
        match integer_literal {
            Expression::IntegerLiteral(integer) => {
                if integer.value != value {
                    eprintln!("integer.value not {value}, got {}", integer.value);
                    return false;
                }

                if integer.token_literal() != value.to_string() {
                    eprintln!(
                        "integer.token_literal not {value}, got {}",
                        integer.token_literal()
                    );
                    return false;
                }
                true
            }
            _ => false,
        }
    }

    fn check_parser_errors(parser: &Parser) {
        let errors = parser.errors();

        if errors.is_empty() {
            return;
        }

        eprintln!("parser has {} errors", errors.len());
        for e in &errors {
            eprintln!("parser error: {e}");
        }

        panic!()
    }
}
