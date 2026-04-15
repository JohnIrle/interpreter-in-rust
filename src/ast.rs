// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT
use crate::token::Token;

pub trait Node {
    fn token_literal(&self) -> String;
}

pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Expression,
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
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
}

pub enum Statement {
    Let(LetStatement),
}

impl Node for Statement {
    fn token_literal(&self) -> String {
        match self {
            Self::Let(s) => s.token_literal(),
        }
    }
}

pub enum Expression {
    Identifier(Identifier),
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
}
