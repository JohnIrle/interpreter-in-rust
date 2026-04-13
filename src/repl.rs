// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::lexer::Lexer;
use crate::token::TokenType;
use std::io::{self, BufRead, BufReader, Read, Write};

pub fn start<R: Read, W: Write>(input: R, mut output: W) -> io::Result<()> {
    let mut reader = BufReader::new(input);

    loop {
        print!(">> ");
        output.flush()?;

        let mut line = String::new();
        reader.read_line(&mut line)?;

        if line.trim().is_empty() {
            continue;
        }

        if line.trim() == "exit" {
            break;
        }

        let mut lex = Lexer::new(line.as_str());

        loop {
            let token = lex.next_token();

            if token.token_type == TokenType::Eof {
                break;
            }

            println!("{token:#?}");
        }
    }

    Ok(())
}
