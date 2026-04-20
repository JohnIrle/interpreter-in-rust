// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT
use crate::token::Token;

pub trait Node {
    fn token_literal(&self) -> String;
    fn string(&self) -> String;
}

#[derive(Debug, Clone)]
pub struct LetStatement {
    pub token: Token,
    pub name: Box<Expression>,
    pub value: Option<Box<Expression>>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();
        out.push_str(&self.token_literal());
        out.push(' ');
        out.push_str(&self.name.string());
        out.push_str(" = ");

        if let Some(value) = &self.value {
            out.push_str(&value.string());
        }

        out.push(';');

        out
    }
}

#[derive(Debug, Clone)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Option<Box<Expression>>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        out.push_str(&self.token_literal());
        out.push(' ');

        if let Some(return_value) = &self.return_value {
            out.push_str(&return_value.string());
        }

        out
    }
}

#[derive(Debug, Clone)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Option<Box<Expression>>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        if let Some(expression) = &self.expression {
            return expression.string();
        }

        String::new()
    }
}

#[derive(Debug, Clone)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<Statement>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        self.statements
            .iter()
            .for_each(|s| out.push_str(&s.string()));

        out
    }
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
}

impl Node for Statement {
    fn token_literal(&self) -> String {
        match self {
            Self::Let(s) => s.token_literal(),
            Self::Return(s) => s.token_literal(),
            Self::Expression(s) => s.token_literal(),
            Self::Block(s) => s.token_literal(),
        }
    }

    fn string(&self) -> String {
        match self {
            Self::Let(s) => s.string(),
            Self::Return(s) => s.string(),
            Self::Expression(s) => s.string(),
            Self::Block(s) => s.string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        self.value.clone()
    }
}

#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        self.value.to_string()
    }
}

#[derive(Debug, Clone)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Box<Expression>,
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        format!("({}{})", self.operator, self.right.string())
    }
}

#[derive(Debug, Clone)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<Expression>,
    pub operator: String,
    pub right: Box<Expression>,
}

impl Node for InfixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        format!(
            "({} {} {})",
            self.left.string(),
            self.operator,
            self.right.string()
        )
    }
}

#[derive(Debug, Clone)]
pub struct Boolean {
    pub token: Token,
    pub value: bool,
}

impl Node for Boolean {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        self.token.literal.clone()
    }
}

#[derive(Debug, Clone)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<Expression>,
    pub consequence: Statement,
    pub alternative: Option<Statement>,
}

impl Node for IfExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();
        out.push_str("if");
        out.push_str(&self.condition.string());
        out.push(' ');
        out.push_str(&self.consequence.string());

        if let Some(alternative) = &self.alternative {
            out.push_str("else ");
            out.push_str(&alternative.string());
        }

        out
    }
}

#[derive(Debug, Clone)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Expression>,
    pub(crate) body: Option<Statement>,
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }

    fn string(&self) -> String {
        let mut out = String::new();

        let params: Vec<String> = self.parameters.iter().map(Node::string).collect();

        out.push_str(&self.token_literal());
        out.push('(');
        out.push_str(&params.join(", "));
        out.push_str(") ");
        if let Some(body) = &self.body {
            out.push_str(&body.string());
        }

        out
    }
}

#[derive(Debug, Clone)]
pub enum Expression {
    Identifier(Identifier),
    IntegerLiteral(IntegerLiteral),
    Prefix(Box<PrefixExpression>),
    Infix(Box<InfixExpression>),
    Boolean(Boolean),
    If(IfExpression),
    FunctionLiteral(FunctionLiteral),
}

impl Node for Expression {
    fn token_literal(&self) -> String {
        match self {
            Self::Identifier(identifier) => identifier.token_literal(),
            Self::IntegerLiteral(integer_literal) => integer_literal.token_literal(),
            Self::Prefix(prefix_expression) => prefix_expression.token_literal(),
            Self::Infix(infix_expression) => infix_expression.token_literal(),
            Self::Boolean(boolean) => boolean.token_literal(),
            Self::If(if_expression) => if_expression.token_literal(),
            Self::FunctionLiteral(function_literal) => function_literal.token_literal(),
        }
    }
    fn string(&self) -> String {
        match self {
            Self::Identifier(identifier) => identifier.string(),
            Self::IntegerLiteral(integer_literal) => integer_literal.string(),
            Self::Prefix(prefix_expression) => prefix_expression.string(),
            Self::Infix(infix_expression) => infix_expression.string(),
            Self::Boolean(boolean) => boolean.string(),
            Self::If(if_expression) => if_expression.string(),
            Self::FunctionLiteral(function_literal) => function_literal.string(),
        }
    }
}

pub struct Program {
    pub statements: Vec<Statement>,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if !self.statements.is_empty() {
            return self.statements[0].token_literal();
        }
        String::new()
    }

    fn string(&self) -> String {
        self.statements.iter().map(Node::string).collect::<String>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn test_string() {
        let program = Program {
            statements: vec![Statement::Let(LetStatement {
                token: Token::new(TokenType::Let, "let".into()),
                name: Box::from(Expression::Identifier(Identifier {
                    token: Token::new(TokenType::Ident, "myVar".into()),
                    value: "myVar".to_string(),
                })),
                value: Some(Box::from(Expression::Identifier(Identifier {
                    token: Token::new(TokenType::Ident, "anotherVar".into()),
                    value: "anotherVar".to_string(),
                }))),
            })],
        };

        assert_eq!(program.string(), "let myVar = anotherVar;");
    }
}
