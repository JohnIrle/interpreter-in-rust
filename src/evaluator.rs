// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::ast::{BlockStatement, Expression, IfExpression, Program, Statement};
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

        if let Some(Object::Return(value)) = result {
            return Some(*value);
        }
    }

    result
}

fn eval_statement(statement: &Statement) -> Option<Object> {
    match statement {
        Statement::Expression(expression_statement) => expression_statement
            .expression
            .as_ref()
            .and_then(|expression| eval_expression(expression)),
        Statement::Block(block_statement) => eval_block_statement(block_statement),
        Statement::Return(return_statement) => return_statement
            .return_value
            .as_ref()
            .and_then(|return_value| eval_expression(return_value))
            .map(|val| Object::Return(Box::new(val))),
        Statement::Let(_) => None,
    }
}

fn eval_expression(expression: &Expression) -> Option<Object> {
    match expression {
        Expression::IntegerLiteral(integer_literal) => Some(Object::Integer(integer_literal.value)),
        Expression::Boolean(boolean) => Some(if boolean.value { TRUE } else { FALSE }),
        Expression::Prefix(expression) => {
            let right = eval_expression(&expression.right);
            Some(eval_prefix_expression(&expression.operator, right.as_ref()))
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
        Expression::If(if_expression) => eval_if_expression(if_expression),
        _ => None,
    }
}

fn eval_prefix_expression(operator: &str, right: Option<&Object>) -> Object {
    match operator {
        "!" => eval_bang_operator_expression(right),
        "-" => eval_minus_prefix_operator_expression(right),
        _ => NULL,
    }
}

const fn eval_bang_operator_expression(right: Option<&Object>) -> Object {
    match right {
        Some(Object::Boolean(false) | Object::Null) => TRUE,
        _ => FALSE,
    }
}

fn eval_minus_prefix_operator_expression(right: Option<&Object>) -> Object {
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
        _ => Object::Null,
    }
}

fn eval_block_statement(block_statement: &BlockStatement) -> Option<Object> {
    let mut result = None;

    for statement in &block_statement.statements {
        result = eval_statement(statement);

        if let Some(Object::Return(_)) = result {
            return result;
        }
    }

    result
}

fn eval_if_expression(if_expression: &IfExpression) -> Option<Object> {
    let condition = eval_expression(&if_expression.condition);

    if is_truthy(condition.as_ref()) {
        return eval_statement(&if_expression.consequence);
    } else if let Some(ref alternative) = if_expression.alternative {
        return eval_statement(alternative);
    }

    Some(NULL)
}

const fn is_truthy(object: Option<&Object>) -> bool {
    !matches!(object, Some(&NULL | &FALSE))
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
            test_integer_object(evaluated.as_ref(), expected);
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
            test_boolean_object(evaluated.as_ref(), expected);
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
            test_boolean_object(evaluated.as_ref(), expected);
        }
    }

    #[test]
    fn test_if_else_expressions() {
        let tests = [
            ("if (true) { 10 }", Some(10)),
            ("if (false) { 10 }", None),
            ("if (1) { 10 }", Some(10)),
            ("if (1 < 2) { 10 }", Some(10)),
            ("if (1 > 2) { 10 }", None),
            ("if (1 > 2) { 10 } else { 20 }", Some(20)),
            ("if (1 < 2) { 10 } else { 20 }", Some(10)),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);

            match expected {
                Some(expected_value) => {
                    test_integer_object(evaluated.as_ref(), expected_value);
                }
                None => {
                    test_null_object(evaluated.as_ref());
                }
            }
        }
    }

    #[test]
    fn test_return_statement() {
        let tests = [
            ("return 10;", 10),
            ("return 10; 9;", 10),
            ("return 2 * 5; 9;", 10),
            ("9; return 2 * 5; 9;", 10),
            ("if (10 > 1) { if (10 > 1) { return 10; } return 1;}", 10),
        ];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(evaluated.as_ref(), expected);
        }
    }

    fn test_eval(input: &str) -> Option<Object> {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();

        eval(&program)
    }

    fn test_integer_object(obj: Option<&Object>, expected: i64) {
        assert_eq!(obj, Some(&Object::Integer(expected)));
    }

    fn test_boolean_object(obj: Option<&Object>, expected: bool) {
        assert_eq!(obj, Some(if expected { &TRUE } else { &FALSE }));
    }

    fn test_null_object(object: Option<&Object>) {
        assert_eq!(object, Some(&NULL), "object is not NULL");
    }
}
