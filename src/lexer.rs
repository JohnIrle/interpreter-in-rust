// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::token::{Token, TokenType};

pub struct Lexer {
    input: String,
    position: usize,
    read_position: usize,
    ch: Option<char>,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        let mut lex = Self {
            input: String::from(input),
            position: 0,
            read_position: 0,
            ch: None,
        };
        lex.read_char();
        lex
    }

    fn read_char(&mut self) {
        if self.read_position >= self.input.len() {
            self.ch = None;
        } else {
            self.ch = self.input.chars().nth(self.read_position);
        }

        self.position = self.read_position;

        self.read_position += 1;
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        let token = match self.ch {
            Some('=') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    Token::new(TokenType::Eq, "==".into())
                } else {
                    Token::new(TokenType::Assign, "=".into())
                }
            }
            Some('+') => Token::new(TokenType::Plus, "+".into()),
            Some('-') => Token::new(TokenType::Minus, "-".into()),
            Some('!') => {
                if self.peek_char() == Some('=') {
                    self.read_char();
                    Token::new(TokenType::NotEq, "!=".into())
                } else {
                    Token::new(TokenType::Bang, "!".into())
                }
            }
            Some('/') => Token::new(TokenType::Slash, "/".into()),
            Some('*') => Token::new(TokenType::Asterisk, "*".into()),
            Some('<') => Token::new(TokenType::Lt, "<".into()),
            Some('>') => Token::new(TokenType::Gt, ">".into()),
            Some(',') => Token::new(TokenType::Comma, ",".into()),
            Some(';') => Token::new(TokenType::SemiColon, ";".into()),
            Some('(') => Token::new(TokenType::LParen, "(".into()),
            Some(')') => Token::new(TokenType::RParen, ")".into()),
            Some('{') => Token::new(TokenType::LBrace, "{".into()),
            Some('}') => Token::new(TokenType::RBrace, "}".into()),
            Some(ch) if is_letter(Some(&ch)) => {
                let literal = self.read_identifier();
                let token_type = TokenType::lookup_ident(literal.as_str());
                return Token::new(token_type, literal);
            }
            Some(ch) if is_digit(Some(&ch)) => {
                let literal = self.read_number();
                return Token::new(TokenType::Int, literal);
            }
            Some(ch) => Token::new(TokenType::Illegal, String::from(ch)),
            None => Token::new(TokenType::Eof, String::new()),
        };

        self.read_char();

        token
    }

    fn read_identifier(&mut self) -> String {
        let position = self.position;

        while is_letter(self.ch.as_ref()) {
            self.read_char();
        }

        let word_length = self.position - position;
        self.input
            .chars()
            .skip(position)
            .take(word_length)
            .collect()
    }

    fn read_number(&mut self) -> String {
        let position = self.position;
        while is_digit(self.ch.as_ref()) {
            self.read_char();
        }

        let digit_length = self.position - position;
        self.input
            .chars()
            .skip(position)
            .take(digit_length)
            .collect()
    }

    fn skip_whitespace(&mut self) {
        while self.ch == Some(' ')
            || self.ch == Some('\t')
            || self.ch == Some('\n')
            || self.ch == Some('\r')
        {
            self.read_char();
        }
    }

    fn peek_char(&self) -> Option<char> {
        if self.read_position >= self.input.len() {
            return None;
        }
        self.input.chars().nth(self.read_position)
    }
}

fn is_letter(ch: Option<&char>) -> bool {
    ch.is_none_or(|char| char.is_alphabetic() || *char == '_')
}

fn is_digit(ch: Option<&char>) -> bool {
    ch.is_none_or(char::is_ascii_digit)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::token::TokenType;

    #[test]
    fn test_next_token() {
        let input = r"let five = 5;
let ten = 10;
let add = fn(x, y) {
    x + y;
};
let result = add(five, ten);
!-/*5;
5 < 10 > 5;

if (5 < 10) {
    return true;
} else {
    return false;
}

10 == 10;
10 != 9;
";

        let tests = [
            (TokenType::Let, "let"),
            (TokenType::Ident, "five"),
            (TokenType::Assign, "="),
            (TokenType::Int, "5"),
            (TokenType::SemiColon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "ten"),
            (TokenType::Assign, "="),
            (TokenType::Int, "10"),
            (TokenType::SemiColon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "add"),
            (TokenType::Assign, "="),
            (TokenType::Function, "fn"),
            (TokenType::LParen, "("),
            (TokenType::Ident, "x"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "y"),
            (TokenType::RParen, ")"),
            (TokenType::LBrace, "{"),
            (TokenType::Ident, "x"),
            (TokenType::Plus, "+"),
            (TokenType::Ident, "y"),
            (TokenType::SemiColon, ";"),
            (TokenType::RBrace, "}"),
            (TokenType::SemiColon, ";"),
            (TokenType::Let, "let"),
            (TokenType::Ident, "result"),
            (TokenType::Assign, "="),
            (TokenType::Ident, "add"),
            (TokenType::LParen, "("),
            (TokenType::Ident, "five"),
            (TokenType::Comma, ","),
            (TokenType::Ident, "ten"),
            (TokenType::RParen, ")"),
            (TokenType::SemiColon, ";"),
            (TokenType::Bang, "!"),
            (TokenType::Minus, "-"),
            (TokenType::Slash, "/"),
            (TokenType::Asterisk, "*"),
            (TokenType::Int, "5"),
            (TokenType::SemiColon, ";"),
            (TokenType::Int, "5"),
            (TokenType::Lt, "<"),
            (TokenType::Int, "10"),
            (TokenType::Gt, ">"),
            (TokenType::Int, "5"),
            (TokenType::SemiColon, ";"),
            (TokenType::If, "if"),
            (TokenType::LParen, "("),
            (TokenType::Int, "5"),
            (TokenType::Lt, "<"),
            (TokenType::Int, "10"),
            (TokenType::RParen, ")"),
            (TokenType::LBrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::True, "true"),
            (TokenType::SemiColon, ";"),
            (TokenType::RBrace, "}"),
            (TokenType::Else, "else"),
            (TokenType::LBrace, "{"),
            (TokenType::Return, "return"),
            (TokenType::False, "false"),
            (TokenType::SemiColon, ";"),
            (TokenType::RBrace, "}"),
            (TokenType::Int, "10"),
            (TokenType::Eq, "=="),
            (TokenType::Int, "10"),
            (TokenType::SemiColon, ";"),
            (TokenType::Int, "10"),
            (TokenType::NotEq, "!="),
            (TokenType::Int, "9"),
            (TokenType::SemiColon, ";"),
            (TokenType::Eof, ""),
        ];

        let mut l = Lexer::new(input);

        for (i, (expected_type, expected_literal)) in tests.iter().enumerate() {
            let tok = l.next_token();

            assert_eq!(
                tok.token_type, *expected_type,
                "tests[{i}] - token_type wrong."
            );
            assert_eq!(
                tok.literal, *expected_literal,
                "tests[{i}] - token_type wrong.",
            );
        }
    }
}
