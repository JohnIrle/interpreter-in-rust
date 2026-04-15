// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub token_type: TokenType,
    pub literal: String,
}

impl Token {
    pub const fn new(token_type: TokenType, literal: String) -> Self {
        Self {
            token_type,
            literal,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenType {
    Illegal,
    Eof,
    Ident,
    Int,
    Assign,
    Plus,
    Minus,
    Bang,
    Asterisk,
    Slash,
    Lt,
    Gt,
    Comma,
    SemiColon,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Eq,
    NotEq,
    Function,
    Let,
    True,
    False,
    If,
    Else,
    Return,
}

impl TokenType {
    pub fn lookup_ident(value: &str) -> Self {
        match value {
            "let" => Self::Let,
            "fn" => Self::Function,
            "true" => Self::True,
            "false" => Self::False,
            "if" => Self::If,
            "else" => Self::Else,
            "return" => Self::Return,
            _ => Self::Ident,
        }
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::Eof => "EOF",
            Self::Ident => "IDENT",
            Self::Int => "INT",
            Self::Assign => "=",
            Self::Plus => "+",
            Self::Minus => "-",
            Self::Bang => "!",
            Self::Asterisk => "*",
            Self::Slash => "/",
            Self::Lt => "<",
            Self::Gt => ">",
            Self::Comma => ",",
            Self::SemiColon => ";",
            Self::LParen => "(",
            Self::RParen => ")",
            Self::LBrace => "{",
            Self::RBrace => "}",
            Self::Eq => "==",
            Self::NotEq => "!=",
            Self::Function => "FUNCTION",
            Self::Let => "LET",
            Self::True => "TRUE",
            Self::False => "FALSE",
            Self::If => "IF",
            Self::Else => "ELSE",
            Self::Return => "RETURN",
            Self::Illegal => "ILLEGAL",
        };
        write!(f, "{s}")
    }
}
