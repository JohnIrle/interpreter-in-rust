// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::ast::{Expression, Program, Statement};
use crate::object::Object;

const TRUE: Object = Object::Boolean(true);
const FALSE: Object = Object::Boolean(false);
const NULL: Object = Object::Null;

pub fn eval(program: &Program) -> Option<Object> {
    eval_program(program)
}

fn eval_program(program: &Program) -> Option<Object> {
    let mut result = None;

    for statement in &program.statements {
        result = eval_statement(statement);
    }

    result
}

fn eval_statement(statement: &Statement) -> Option<Object> {
    match statement {
        Statement::Expression(expression_statement) => expression_statement
            .expression
            .as_ref()
            .and_then(|expression| eval_expression(expression)),
        _ => None,
    }
}

fn eval_expression(expression: &Expression) -> Option<Object> {
    match expression {
        Expression::IntegerLiteral(integer_literal) => Some(Object::Integer(integer_literal.value)),
        Expression::Boolean(boolean) => Some(if boolean.value { TRUE } else { FALSE }),
        Expression::Prefix(expression) => {
            let right = eval_expression(&expression.right);
            Some(eval_prefix_expression(&expression.operator, right))
        }
        _ => None,
    }
}

fn eval_prefix_expression(operator: &str, right: Option<Object>) -> Object {
    match operator {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => NULL,
    }
}

const fn eval_bang_operator_expression(right: Option<Object>) -> Object {
    match right {
        Some(Object::Boolean(true)) => FALSE,
        Some(Object::Boolean(false) | Object::Null) => TRUE,
        #[allow(clippy::match_same_arms)]
        _ => FALSE,
    }
}

const fn eval_minus_prefix_operator_expression(right: Option<Object>) -> Object {
    match right {
        Some(Object::Integer(value)) => Object::Integer(-value),
        _ => NULL,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;
    use crate::object::Object;
    use crate::parser::Parser;

    #[test]
    fn test_eval_integer_expression() {
        let tests = [("5", 5), ("10", 10), ("-5", -5), ("-10", -10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(evaluated, expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = [("true", true), ("false", false)];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_boolean_object(evaluated, expected);
        }
    }

    #[test]
    fn test_bang_operator() {
        let tests = [
            ("!true", false),
            ("!false", true),
            ("!5", false),
            ("!!true", true),
            ("!!false", false),
            ("!!5", true),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_boolean_object(evaluated, expected);
        }
    }

    fn test_eval(input: &str) -> Option<Object> {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();

        eval(&program)
    }

    fn test_integer_object(obj: Option<Object>, expected: i64) {
        assert_eq!(obj, Some(Object::Integer(expected)));
    }

    fn test_boolean_object(obj: Option<Object>, expected: bool) {
        assert_eq!(obj, Some(if expected { TRUE } else { FALSE }));
    }
}
