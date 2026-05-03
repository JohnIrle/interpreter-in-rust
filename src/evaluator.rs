// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::ast::{Expression, Program, Statement};
use crate::object::Object;
use crate::object::Object::Null;

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
        Expression::Infix(infix_expression) => {
            let left = eval_expression(&infix_expression.left);
            let right = eval_expression(&infix_expression.right);
            Some(eval_infix_expression(
                &infix_expression.operator,
                left,
                right,
            ))
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

fn eval_infix_expression(operator: &str, left: Option<Object>, right: Option<Object>) -> Object {
    match (left, right) {
        (Some(Object::Integer(left)), Some(Object::Integer(right))) => {
            eval_integer_infix_expression(operator, left, right)
        }
        (Some(left_obj), Some(right_obj)) if operator == "==" => {
            Object::Boolean(left_obj == right_obj)
        }
        (Some(left_obj), Some(right_obj)) if operator == "!=" => {
            Object::Boolean(left_obj != right_obj)
        }
        _ => NULL,
    }
}

fn eval_integer_infix_expression(operator: &str, left: i64, right: i64) -> Object {
    match operator {
        "+" => Object::Integer(left + right),
        "-" => Object::Integer(left - right),
        "*" => Object::Integer(left * right),
        "/" => Object::Integer(left / right),
        "<" => Object::Boolean(left < right),
        ">" => Object::Boolean(left > right),
        "==" => Object::Boolean(left == right),
        "!=" => Object::Boolean(left != right),
        _ => Null,
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
        let tests = [
            ("5", 5),
            ("10", 10),
            ("-5", -5),
            ("-10", -10),
            ("5 + 5 + 5 + 5 - 10", 10),
            ("2 * 2 * 2 * 2 * 2", 32),
            ("-50 + 100 + -50", 0),
            ("5 * 2 + 10", 20),
            ("5 + 2 * 10", 25),
            ("20 + 2 * -10", 0),
            ("50 / 2 * 2 + 10", 60),
            ("2 * (5 + 10)", 30),
            ("3 * 3 * 3 + 10", 37),
            ("3 * (3 * 3) + 10", 37),
            ("(5 + 10 * 2 + 15 / 3) * 2 + -10", 50),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(evaluated, expected);
        }
    }

    #[test]
    fn test_eval_boolean_expression() {
        let tests = [
            ("true", true),
            ("false", false),
            ("1 < 2", true),
            ("1 > 2", false),
            ("1 < 1", false),
            ("1 > 1", false),
            ("1 == 1", true),
            ("1 != 1", false),
            ("1 == 2", false),
            ("1 != 2", true),
            ("true == true", true),
            ("false == false", true),
            ("true == false", false),
            ("true != false", true),
            ("false != true", true),
            ("(1 < 2) == true", true),
            ("(1 < 2) == false", false),
            ("(1 > 2) == true", false),
            ("(1 > 2) == false", true),
        ];

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
