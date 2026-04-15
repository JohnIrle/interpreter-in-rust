// SPDX-FileCopyrightText: 2026 John Irle
//
// SPDX-License-Identifier: MIT

mod ast;
mod lexer;
mod parser;
mod repl;
mod token;

fn main() -> std::io::Result<()> {
    println!("This is the Monkey programming language!");
    println!("Feel free to type in commands. Type 'exit' to quit.");

    repl::start(std::io::stdin(), std::io::stdout())
}
