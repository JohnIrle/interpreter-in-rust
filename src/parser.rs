// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::ast::{
    BlockStatement, Boolean, CallExpression, Expression, ExpressionStatement, FunctionLiteral,
    Identifier, IfExpression, InfixExpression, IntegerLiteral, LetStatement, PrefixExpression,
    Program, ReturnStatement, Statement,
};
use crate::lexer::Lexer;
use crate::parser::Precedence::Lowest;
use crate::token::{Token, TokenType};
use std::collections::HashMap;

type PrefixParseFn<'a> = fn(&mut Parser<'a>) -> Option<Expression>;
type InfixParseFn<'a> = fn(&mut Parser<'a>, Expression) -> Option<Expression>;

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

const fn precedences(token_type: &TokenType) -> Option<Precedence> {
    match token_type {
        TokenType::Eq | TokenType::NotEq => Some(Precedence::Equals),
        TokenType::Lt | TokenType::Gt => Some(Precedence::LessGreater),
        TokenType::Plus | TokenType::Minus => Some(Precedence::Sum),
        TokenType::Slash | TokenType::Asterisk => Some(Precedence::Product),
        TokenType::LParen => Some(Precedence::Call),
        _ => None,
    }
}

pub struct Parser<'a> {
    lexer: &'a mut Lexer,
    current_token: Option<Token>,
    peek_token: Option<Token>,
    errors: Vec<String>,

    prefix_parse_fns: HashMap<TokenType, PrefixParseFn<'a>>,
    infix_parse_fns: HashMap<TokenType, InfixParseFn<'a>>,
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
        parser.register_infix(TokenType::Plus, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Minus, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Slash, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Asterisk, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Eq, Parser::parse_infix_expression);
        parser.register_infix(TokenType::NotEq, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Lt, Parser::parse_infix_expression);
        parser.register_infix(TokenType::Gt, Parser::parse_infix_expression);
        parser.register_prefix(TokenType::True, Parser::parse_boolean);
        parser.register_prefix(TokenType::False, Parser::parse_boolean);
        parser.register_prefix(TokenType::LParen, Parser::parse_grouped_expression);
        parser.register_prefix(TokenType::If, Parser::parse_if_expression);
        parser.register_prefix(TokenType::Function, Parser::parse_function_literal);
        parser.register_infix(TokenType::LParen, Parser::parse_call_expression);

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

    fn register_infix(&mut self, token_type: TokenType, function: InfixParseFn<'a>) {
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
            name: Box::new(Expression::Identifier(name)),
            value: value.map(Box::new),
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
            return_value: return_value.map(Box::new),
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
            expression: expression.map(Box::new),
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

    fn parse_boolean(&mut self) -> Option<Expression> {
        let token = self.current_token.clone()?;
        Some(Expression::Boolean(Boolean {
            token,
            value: self.cur_token_is(&TokenType::True),
        }))
    }

    fn parse_grouped_expression(&mut self) -> Option<Expression> {
        self.next_token();

        let expression = self.parse_expression(&Lowest);

        if !self.expect_peek(&TokenType::RParen) {
            return None;
        }

        expression
    }

    fn no_prefix_parse_fn_error(&mut self, token_type: &TokenType) {
        self.errors
            .push(format!("no prefix parse function for {token_type} found"));
    }

    fn parse_expression(&mut self, precedence: &Precedence) -> Option<Expression> {
        let token_type = &self.current_token.as_ref()?.token_type.clone();
        let prefix = self.prefix_parse_fns.get(token_type).copied();

        if prefix.is_none() {
            self.no_prefix_parse_fn_error(token_type);
            return None;
        }

        let mut left_exp = prefix?(self)?;

        while !self.peek_token_is(&TokenType::SemiColon) && precedence < &self.peek_precedence() {
            let peek_token = &self.peek_token.as_ref()?.token_type;
            let infix = self.infix_parse_fns.get(peek_token).copied();
            if infix.is_none() {
                return Some(left_exp);
            }

            self.next_token();
            left_exp = infix?(self, left_exp)?;
        }

        Some(left_exp)
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

    fn parse_infix_expression(&mut self, left: Expression) -> Option<Expression> {
        let token = self.current_token.clone()?;
        let operator = token.literal.clone();
        let precedence = self.cur_precedence();

        self.next_token();

        let right = self.parse_expression(&precedence)?;

        Some(Expression::Infix(Box::from(InfixExpression {
            token,
            operator,
            left: Box::from(left),
            right: Box::from(right),
        })))
    }

    fn parse_if_expression(&mut self) -> Option<Expression> {
        let token = self.current_token.clone()?;

        if !self.expect_peek(&TokenType::LParen) {
            return None;
        }

        self.next_token();
        let condition = self.parse_expression(&Lowest)?;

        if !self.expect_peek(&TokenType::RParen) {
            return None;
        }

        if !self.expect_peek(&TokenType::LBrace) {
            return None;
        }

        let consequence = self.parse_block_statement()?;

        let alternative = if self.peek_token_is(&TokenType::Else) {
            self.next_token();

            if !self.expect_peek(&TokenType::LBrace) {
                return None;
            }

            self.parse_block_statement()
        } else {
            None
        };

        Some(Expression::If(IfExpression {
            token,
            condition: Box::new(condition),
            consequence,
            alternative,
        }))
    }

    fn parse_block_statement(&mut self) -> Option<Statement> {
        let token = self.current_token.clone()?;
        let mut statements = Vec::new();

        self.next_token();

        while !self.cur_token_is(&TokenType::RBrace) && !self.cur_token_is(&TokenType::Eof) {
            let statement = self.parse_statement();
            if let Some(statement) = statement {
                statements.push(statement);
            }

            self.next_token();
        }

        Some(Statement::Block(BlockStatement { token, statements }))
    }

    fn parse_function_literal(&mut self) -> Option<Expression> {
        let token = self.current_token.clone()?;
        if !self.expect_peek(&TokenType::LParen) {
            return None;
        }

        let parameters = self.parse_function_parameters();

        if !self.expect_peek(&TokenType::LBrace) {
            return None;
        }

        let body = self.parse_block_statement();

        Some(Expression::FunctionLiteral(FunctionLiteral {
            token,
            parameters,
            body,
        }))
    }

    fn parse_function_parameters(&mut self) -> Vec<Expression> {
        let mut identifiers = Vec::new();

        if self.peek_token_is(&TokenType::RParen) {
            self.next_token();
            return identifiers;
        }

        self.next_token();

        let Some(token) = self.current_token.clone() else {
            return identifiers;
        };

        let ident = Expression::Identifier(Identifier {
            token: token.clone(),
            value: token.literal,
        });
        identifiers.push(ident);

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();

            let Some(token) = self.current_token.clone() else {
                return identifiers;
            };

            let ident = Expression::Identifier(Identifier {
                token: token.clone(),
                value: token.literal,
            });

            identifiers.push(ident);
        }

        if !self.expect_peek(&TokenType::RParen) {
            return Vec::new();
        }

        identifiers
    }

    fn parse_call_expression(&mut self, function: Expression) -> Option<Expression> {
        let token = self.current_token.clone()?;

        let arguments = self.parse_call_arguments();

        Some(Expression::Call(CallExpression {
            token,
            function: Box::from(function),
            arguments,
        }))
    }

    fn parse_call_arguments(&mut self) -> Vec<Expression> {
        let mut args = Vec::new();

        if self.peek_token_is(&TokenType::RParen) {
            self.next_token();
            return args;
        }

        self.next_token();

        let Some(expression) = self.parse_expression(&Lowest) else {
            return args;
        };
        args.push(expression);

        while self.peek_token_is(&TokenType::Comma) {
            self.next_token();
            self.next_token();

            let Some(expression) = self.parse_expression(&Lowest) else {
                return args;
            };
            args.push(expression);
        }

        if !self.expect_peek(&TokenType::RParen) {
            return Vec::new();
        }

        args
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

    fn peek_precedence(&self) -> Precedence {
        let token = self.peek_token.clone();
        if let Some(token) = token
            && let Some(precedence) = precedences(&token.token_type)
        {
            return precedence;
        }
        Lowest
    }

    fn cur_precedence(&self) -> Precedence {
        let token = self.current_token.clone();
        if let Some(token) = token
            && let Some(precedence) = precedences(&token.token_type)
        {
            return precedence;
        }
        Lowest
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
    use crate::ast::Statement;

    #[test]
    fn test_let_statements() {
        let tests = [
            ("let x = 5;", "x", Expected::from(5)),
            ("let y = true;", "y", Expected::from(true)),
            ("let foobar = y;", "foobar", Expected::from("y")),
        ];

        for (input, expected_identifier, expected_value) in tests {
            let mut lexer = Lexer::new(input);
            let mut parser = Parser::new(&mut lexer);

            let program = parser.parse_program();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            let statement = &program.statements[0];
            test_let_statement(statement, expected_identifier);

            if let Statement::Let(let_statement) = statement {
                if let Some(expression) = &let_statement.value {
                    test_literal_expression(expression, &expected_value);
                }
            } else {
                panic!("statement not let statement");
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
            let Statement::Return(return_stmt) = statement else {
                panic!("stmt not ReturnStatement");
            };
            assert_eq!(
                return_stmt.token_literal(),
                "return",
                "returnStmt.TokenLiteral not 'return' got {}",
                return_stmt.token_literal()
            );
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

        let Statement::Expression(statement) = &program.statements[0] else {
            panic!("program.statements[0] is not ExpressionStatement");
        };

        let Some(expression) = &statement.expression else {
            panic!("expression is None");
        };

        let Expression::Identifier(ident) = expression.as_ref() else {
            panic!("Expression is not Identifier");
        };

        let value = &ident.value;
        assert_eq!(value, "foobar", "ident value not foobar, got {value}");

        let token_literal = ident.token_literal();
        assert_eq!(
            token_literal, "foobar",
            "ident token_literal not foobar, got {token_literal}"
        );
    }

    #[test]
    fn test_integer_literal_expression() {
        let input = "5;";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = (parser).parse_program();

        check_parser_errors(&parser);

        assert_eq!(program.statements.len(), 1);

        let Statement::Expression(statement) = &program.statements[0] else {
            panic!("program.statements[0] is not ExpressionStatement");
        };

        let Some(expression) = &statement.expression else {
            panic!("expression is None");
        };

        let Expression::IntegerLiteral(integer_literal) = expression.as_ref() else {
            panic!("Expression is not Integer literal");
        };

        let value = &integer_literal.value;
        assert_eq!(*value, 5, "literal value  not 5 got {value}");

        let token_literal = integer_literal.token_literal();
        assert_eq!(
            token_literal, "5",
            "token literal is not '5' got {token_literal}"
        );
    }

    #[test]
    fn test_parsing_prefix_expressions() {
        let tests = [
            ("!5;", "!", Expected::from(5)),
            ("-15;", "-", Expected::from(15)),
            ("!true;", "!", Expected::from(true)),
            ("!false;", "!", Expected::from(false)),
        ];

        for (input, expected_operator, expected_value) in tests {
            let mut lexer = Lexer::new(input);
            let mut parser = Parser::new(&mut lexer);

            let program = parser.parse_program();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            let Statement::Expression(statement) = &program.statements[0] else {
                panic!("program.statements[0] is not ExpressionStatement");
            };

            let Some(expression) = &statement.expression else {
                panic!("expression is None");
            };

            let Expression::Prefix(expression) = expression.as_ref() else {
                panic!("Expression is not prefix expression");
            };

            let exp_operator = &expression.operator;
            assert_eq!(exp_operator, expected_operator);

            test_literal_expression(&expression.right, &expected_value);
        }
    }

    #[test]
    fn test_parsing_infix_expressions() {
        let tests = [
            ("5 + 5;", Expected::from(5), "+", Expected::from(5)),
            ("5 - 5;", Expected::from(5), "-", Expected::from(5)),
            ("5 * 5;", Expected::from(5), "*", Expected::from(5)),
            ("5 / 5;", Expected::from(5), "/", Expected::from(5)),
            ("5 > 5;", Expected::from(5), ">", Expected::from(5)),
            ("5 < 5;", Expected::from(5), "<", Expected::from(5)),
            ("5 == 5;", Expected::from(5), "==", Expected::from(5)),
            ("5 != 5;", Expected::from(5), "!=", Expected::from(5)),
            (
                "true == true",
                Expected::from(true),
                "==",
                Expected::from(true),
            ),
        ];

        for (input, expected_left_value, expected_operator, expected_right_value) in &tests {
            let mut lexer = Lexer::new(input);
            let mut parser = Parser::new(&mut lexer);

            let program = parser.parse_program();
            check_parser_errors(&parser);

            assert_eq!(program.statements.len(), 1);

            let Statement::Expression(statement) = &program.statements[0] else {
                panic!("program.statements[0] is not ExpressionStatement");
            };

            let Some(expression) = &statement.expression else {
                panic!("expression is None");
            };

            let Expression::Infix(expression) = expression.as_ref() else {
                panic!("Expression is not infix expression");
            };

            test_infix_expression(
                expression,
                expected_left_value,
                expected_operator,
                expected_right_value,
            );
        }
    }

    #[test]
    fn test_operator_precedence_parsing() {
        let tests = [
            ("true", "true"),
            ("false", "false"),
            ("3 > 5 == false", "((3 > 5) == false)"),
            ("3 < 5 == true", "((3 < 5) == true)"),
            ("-a * b", "((-a) * b)"),
            ("!-a", "(!(-a))"),
            ("a + b + c", "((a + b) + c)"),
            ("a + b - c", "((a + b) - c)"),
            ("a * b * c", "((a * b) * c)"),
            ("a * b / c", "((a * b) / c)"),
            ("a + b / c", "(a + (b / c))"),
            ("a + b * c + d / e - f", "(((a + (b * c)) + (d / e)) - f)"),
            ("3 + 4; -5 * 5", "(3 + 4)((-5) * 5)"),
            ("5 > 4 == 3 < 4", "((5 > 4) == (3 < 4))"),
            ("5 < 4 != 3 > 4", "((5 < 4) != (3 > 4))"),
            (
                "3 + 4 * 5 == 3 * 1 + 4 * 5",
                "((3 + (4 * 5)) == ((3 * 1) + (4 * 5)))",
            ),
            ("1 + (2 + 3) + 4", "((1 + (2 + 3)) + 4)"),
            ("(5 + 5) * 2", "((5 + 5) * 2)"),
            ("2 / (5 + 5)", "(2 / (5 + 5))"),
            ("-(5 + 5)", "(-(5 + 5))"),
            ("!(true == true)", "(!(true == true))"),
            ("a + add(b * c) + d", "((a + add((b * c))) + d)"),
            (
                "add(a, b, 1, 2 * 3, 4 + 5, add(6, 7 * 8))",
                "add(a, b, 1, (2 * 3), (4 + 5), add(6, (7 * 8)))",
            ),
            (
                "add(a + b + c * d / f + g)",
                "add((((a + b) + ((c * d) / f)) + g))",
            ),
        ];

        for (input, expected) in &tests {
            let mut lexer = Lexer::new(input);
            let mut parser = Parser::new(&mut lexer);

            let program = parser.parse_program();
            check_parser_errors(&parser);

            assert_eq!(program.string(), *expected);
        }
    }

    #[test]
    fn test_boolean_expression() {
        let tests = [("true;", true), ("false;", false)];

        for (input, expected) in &tests {
            let mut lexer = Lexer::new(input);
            let mut parser = Parser::new(&mut lexer);

            let program = parser.parse_program();
            check_parser_errors(&parser);

            assert_eq!(
                program.statements.len(),
                1,
                "program does not have 1 statement"
            );

            let Statement::Expression(statement) = &program.statements[0] else {
                panic!("program.statements[0] is not ExpressionStatement");
            };

            let Some(expression) = &statement.expression else {
                panic!("expression is None");
            };

            let Expression::Boolean(expression) = expression.as_ref() else {
                panic!("Expression is not boolean expression");
            };

            assert_eq!(
                expression.value, *expected,
                "boolean value does not match expected"
            );
        }
    }

    #[test]
    fn test_if_expression() {
        let input = "if (x < y) { x }";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            1,
            "program does not have 1 statement"
        );

        let Statement::Expression(statement) = &program.statements[0] else {
            panic!("program.statements[0] is not ExpressionStatement");
        };

        let Some(expression) = &statement.expression else {
            panic!("expression is None");
        };

        let Expression::If(expression) = expression.as_ref() else {
            panic!("expression is not IfExpression");
        };

        let Expression::Infix(condition) = expression.condition.as_ref() else {
            panic!("condition is not InfixExpression");
        };

        test_infix_expression(condition, &Expected::from("x"), "<", &Expected::from("y"));

        let Statement::Block(block) = &expression.consequence else {
            panic!("consequence is not BlockStatement")
        };

        assert_eq!(
            block.statements.len(),
            1,
            "consequence is not 1 statements. got {}",
            block.statements.len()
        );

        let Statement::Expression(expr_stmt) = &block.statements[0] else {
            panic!("block[0] is not ExpressionStatement");
        };

        let Some(expr) = &expr_stmt.expression else {
            panic!("missing expression");
        };

        test_identifier(expr, "x");
    }

    #[test]
    fn test_if_else_expression() {
        let input = "if (x < y) { x } else { y }";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            1,
            "program does not have 1 statement"
        );

        let Statement::Expression(statement) = &program.statements[0] else {
            panic!("program.statements[0] is not ExpressionStatement");
        };

        let Some(expression) = &statement.expression else {
            panic!("expression is None");
        };

        let Expression::If(expression) = expression.as_ref() else {
            panic!("expression is not IfExpression");
        };

        let Expression::Infix(condition) = expression.condition.as_ref() else {
            panic!("condition is not InfixExpression");
        };

        test_infix_expression(condition, &Expected::from("x"), "<", &Expected::from("y"));

        let Statement::Block(block) = &expression.consequence else {
            panic!("consequence is not BlockStatement")
        };

        assert_eq!(
            block.statements.len(),
            1,
            "consequence is not 1 statements. got {}",
            block.statements.len()
        );

        let Statement::Expression(expr_stmt) = &block.statements[0] else {
            panic!("block[0] is not ExpressionStatement");
        };

        let Some(expr) = &expr_stmt.expression else {
            panic!("missing expression");
        };

        test_identifier(expr, "x");

        let Some(Statement::Block(block)) = &expression.alternative else {
            panic!("consequence is not BlockStatement")
        };

        assert_eq!(
            block.statements.len(),
            1,
            "consequence is not 1 statements. got {}",
            block.statements.len()
        );

        let Statement::Expression(expr_stmt) = &block.statements[0] else {
            panic!("block[0] is not ExpressionStatement");
        };

        let Some(expr) = &expr_stmt.expression else {
            panic!("missing expression");
        };

        test_identifier(expr, "y");
    }

    #[test]
    fn test_function_literal_parsing() {
        let input = "fn(x, y) { x + y; }";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);

        let program = parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            1,
            "program body does not contain 1 statements got={}",
            program.statements.len()
        );

        let Statement::Expression(statement) = &program.statements[0] else {
            panic!("program.statements[0] is not ExpressionStatement");
        };

        let Some(Expression::FunctionLiteral(function)) = &statement.expression.as_deref() else {
            panic!("statement.expression is not FunctionLiteral");
        };

        assert_eq!(
            function.parameters.len(),
            2,
            "function parameters wrong. want 2, got={}",
            function.parameters.len()
        );

        test_literal_expression(&function.parameters[0], &Expected::from("x"));
        test_literal_expression(&function.parameters[1], &Expected::from("y"));

        let Some(Statement::Block(block_statement)) = &function.body else {
            panic!("function.body not block statement");
        };
        assert_eq!(
            block_statement.statements.len(),
            1,
            "function.body.statements length not 1 got={}",
            block_statement.statements.len()
        );

        let Statement::Expression(expression_statement) = &block_statement.statements[0] else {
            panic!("function body stmt is not ExpressionStatement");
        };

        let Some(Expression::Infix(infix_expression)) = expression_statement.expression.as_deref()
        else {
            panic!("expression statement not infix expression");
        };

        test_infix_expression(
            infix_expression,
            &Expected::from("x"),
            "+",
            &Expected::from("y"),
        );
    }

    #[test]
    fn test_function_parameter_parsing() {
        let tests = [
            ("fn() {};", Vec::new()),
            ("fn(x) {};", vec!["x"]),
            ("fn(x, y, z) {};", vec!["x", "y", "z"]),
        ];

        for (input, expected_params) in &tests {
            let mut lexer = Lexer::new(input);
            let mut parser = Parser::new(&mut lexer);
            let program = parser.parse_program();
            check_parser_errors(&parser);

            let Statement::Expression(expression_statement) = &program.statements[0] else {
                panic!("program.statements[0] is not ExpressionStatement");
            };

            let Some(Expression::FunctionLiteral(function)) =
                expression_statement.expression.as_deref()
            else {
                panic!("expression_statement.expression not FunctionLiteral");
            };

            assert_eq!(
                function.parameters.len(),
                expected_params.len(),
                "length parameters wrong. want {}, got={}",
                expected_params.len(),
                function.parameters.len()
            );

            for (i, ident) in expected_params.iter().enumerate() {
                test_literal_expression(&function.parameters[i], &Expected::from(*ident));
            }
        }
    }

    #[test]
    fn test_call_expression_parsing() {
        let input = "add(1, 2 * 3, 4 + 5);";

        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();
        check_parser_errors(&parser);

        assert_eq!(
            program.statements.len(),
            1,
            "program.statements does not contain 1 statements, got={}",
            program.statements.len()
        );

        let Statement::Expression(statement) = &program.statements[0] else {
            panic!("stmt is not ExpressionStatement");
        };

        let Some(Expression::Call(call_expression)) = statement.expression.as_deref() else {
            panic!("statement.expression is not CallExpression")
        };

        test_identifier(&call_expression.function, "add");

        assert_eq!(
            call_expression.arguments.len(),
            3,
            "wrong length of arguments. got={}",
            call_expression.arguments.len()
        );

        test_literal_expression(&call_expression.arguments[0], &Expected::from(1));

        let Expression::Infix(infix) = &call_expression.arguments[1] else {
            panic!("call_expression.arguements[1] is not InfixExpression")
        };

        test_infix_expression(infix, &Expected::from(2), "*", &Expected::from(3));

        let Expression::Infix(infix) = &call_expression.arguments[2] else {
            panic!("call_expression.arguements[2] is not InfixExpression")
        };

        test_infix_expression(infix, &Expected::from(4), "+", &Expected::from(5));
    }

    fn test_let_statement(statement: &Statement, name: &str) {
        assert_eq!(
            statement.token_literal(),
            "let",
            "token_literal not 'let. got={}",
            statement.token_literal()
        );

        match statement {
            Statement::Let(let_stmt) => match &let_stmt.name.as_ref() {
                Expression::Identifier(ident) => {
                    assert_eq!(
                        ident.value, name,
                        "let_stmt.name.value not '{}. got={}",
                        name, ident.value
                    );

                    assert_eq!(
                        let_stmt.name.token_literal(),
                        name,
                        "let_stmt.name not '{:#?}. got={:#?}",
                        name,
                        let_stmt.name
                    );
                }
                _ => panic!("let_stmt.name is not Identifier"),
            },
            _ => panic!("statement is not Let"),
        }
    }

    fn test_infix_expression(
        infix_expression: &InfixExpression,
        left: &Expected,
        operator: &str,
        right: &Expected,
    ) {
        test_literal_expression(&infix_expression.left, left);

        assert_eq!(
            infix_expression.operator, operator,
            "expression operator is not {operator}, got {}",
            infix_expression.operator
        );

        test_literal_expression(&infix_expression.right, right);
    }

    fn test_literal_expression(expression: &Expression, expected: &Expected) {
        match expected {
            Expected::Int(v) => test_integer_literal(expression, *v),
            Expected::Str(v) => test_identifier(expression, v),
            Expected::Boolean(v) => test_boolean_literal(expression, *v),
        }
    }

    fn test_integer_literal(integer_literal: &Expression, value: i64) {
        match integer_literal {
            Expression::IntegerLiteral(integer) => {
                assert_eq!(
                    integer.value, value,
                    "integer.value not {value}, got {}",
                    integer.value
                );

                assert_eq!(
                    integer.token_literal(),
                    value.to_string(),
                    "integer.token_literal not {value}, got {}",
                    integer.token_literal()
                );
            }
            _ => panic!("expression not Integer Literal"),
        }
    }

    fn test_identifier(expression: &Expression, value: &str) {
        match expression {
            Expression::Identifier(identifier) => {
                assert_eq!(
                    identifier.value, value,
                    "identifier value not {value}, got {}",
                    identifier.value
                );

                assert_eq!(
                    identifier.token_literal(),
                    value,
                    "identifier token literal not {value}, got {}",
                    identifier.token_literal()
                );
            }
            _ => panic!("expression not Identifier"),
        }
    }

    fn test_boolean_literal(expression: &Expression, value: bool) {
        match expression {
            Expression::Boolean(boolean) => {
                assert_eq!(
                    boolean.value, value,
                    "boolean value not {value}, got {}",
                    boolean.value
                );

                assert_eq!(
                    boolean.token_literal(),
                    value.to_string(),
                    "identifier token literal not {value}, got {}",
                    boolean.token_literal()
                );
            }
            _ => panic!("expression not Boolean"),
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

    pub enum Expected<'a> {
        Int(i64),
        Str(&'a str),
        Boolean(bool),
    }

    impl From<i64> for Expected<'_> {
        fn from(value: i64) -> Self {
            Expected::Int(value)
        }
    }

    impl<'a> From<&'a str> for Expected<'a> {
        fn from(value: &'a str) -> Self {
            Expected::Str(value)
        }
    }

    impl From<bool> for Expected<'_> {
        fn from(value: bool) -> Self {
            Expected::Boolean(value)
        }
    }
}
