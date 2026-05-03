// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

use crate::evaluator;
use crate::lexer::Lexer;
use crate::parser::Parser;
use std::io::{self, BufRead, BufReader, Read, Write};

const PROMPT: &str = ">> ";

pub fn start<R: Read, W: Write>(input: R, mut output: W) -> io::Result<()> {
    let mut reader = BufReader::new(input);

    loop {
        write!(output, "{PROMPT}")?;
        output.flush()?;

        let mut line = String::new();
        let bytes_read = reader.read_line(&mut line)?;

        if bytes_read == 0 {
            return Ok(());
        }

        if line.trim() == "exit" {
            break;
        }

        let mut lex = Lexer::new(line.as_str());
        let mut parser = Parser::new(&mut lex);

        let program = parser.parse_program();
        if !parser.errors().is_empty() {
            print_parser_errors(&mut output, parser.errors())?;
            continue;
        }

        let evaluated = evaluator::eval(&program);
        if let Some(obj) = evaluated {
            writeln!(output, "{}", obj.inspect())?;
        }
    }

    Ok(())
}

const MONKEY_FACE: &str = r#"            __,__
   .--.  .-"     "-.  .--.
  / .. \/  .-. .-.  \/ .. \
 | |  '|  /   Y   \  |'  | |
 | \   \  \ 0 | 0 /  /   / |
  \ '- ,\.-"""""""-./, -' /
   ''-' /_   ^ ^   _\ '-''
       |  \._   _./  |
       \   \ '~' /   /
        '._ '-=-' _.'
           '-----'
"#;

fn print_parser_errors<W: Write>(output: &mut W, errors: Vec<String>) -> io::Result<()> {
    write!(output, "{MONKEY_FACE}")?;
    writeln!(output, "Woops! We ran into some monkey business here!")?;
    writeln!(output, " parser errors:")?;
    for msg in errors {
        writeln!(output, "\t{msg}")?;
    }
    Ok(())
}
