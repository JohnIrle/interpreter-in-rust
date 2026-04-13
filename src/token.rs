// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use std::fmt;

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

#[derive(Debug, PartialEq, Eq)]
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
            Self::Int => "ENT",
            Self::Assign => "ASSIGN",
            Self::Plus => "PLUS",
            Self::Minus => "MINUS",
            Self::Bang => "BANG",
            Self::Asterisk => "ASTERISK",
            Self::Slash => "SLASH",
            Self::Lt => "LT",
            Self::Gt => "GT",
            Self::Comma => "COMMA",
            Self::SemiColon => "SEMICOLON",
            Self::LParen => "LPAREN",
            Self::RParen => "RPAREN",
            Self::LBrace => "LBRAC",
            Self::RBrace => "RBRACE",
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
