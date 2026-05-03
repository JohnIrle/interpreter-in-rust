// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::ast::{Expression, Program, Statement};
use crate::object::Object;

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

const fn eval_expression(expression: &Expression) -> Option<Object> {
    match expression {
        Expression::IntegerLiteral(integer_literal) => Some(Object::Integer(integer_literal.value)),
        _ => None,
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
        let tests = [("5", 5), ("10", 10)];

        for (input, expected) in tests {
            let evaluated = test_eval(input);
            test_integer_object(evaluated, expected);
        }
    }

    fn test_eval(input: &str) -> Option<Object> {
        let mut lexer = Lexer::new(input);
        let mut parser = Parser::new(&mut lexer);
        let program = parser.parse_program();

        eval(&program)
    }

    fn test_integer_object(obj: Option<Object>, expected: i64) {
        if let Some(obj) = obj {
            match obj {
                Object::Integer(value) => assert_eq!(value, expected),
                _ => panic!("Object not Integer"),
            }
        } else {
            panic!("Object is None")
        }
    }
}
