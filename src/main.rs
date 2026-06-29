use std::{fs::File, io::Read};

use crate::{lexer::lexer::Lexer, parser::parser::parse_program};

mod lexer;
mod parser;
mod semantics;

const FILE_PATH: &str = "/home/harry-phillips/Desktop/ccompiler/src/test.c";

fn main() {
    let mut file = File::open(FILE_PATH).unwrap();
    let mut program = String::new();
    file.read_to_string(&mut program).unwrap();

    let mut lexer = Lexer::new(&program).unwrap();

    let parser = parse_program(&mut lexer);
    if let Err(err_msg) = parser {
        write_error_message(&program, FILE_PATH, &err_msg, &lexer);
        return;
    }

    println!("{}", parser.unwrap());
}

fn write_error_message(file: &str, file_name: &str, error_msg: &str, lexer: &Lexer) {
    let char_index = lexer.last_index();
    let mut line_num = 1;
    let mut column_num = 1;
    let mut previous_error_line = String::new();
    let mut error_line = String::new();
    for (i, char) in file.char_indices() {
        if i < char_index {
            column_num += 1;
        }

        if char != '\n' {
            error_line.push(char);
        }

        if i >= char_index && char == '\n' {
            break;
        }

        if char == '\n' {
            line_num += 1;
            column_num = 1;

            previous_error_line = error_line.clone();
            error_line.clear();
        }
    }

    let error_colored = "\x1b[1m\x1b[31merror:\x1b[0m";
    println!(
        "\x1b[1m{file_name}:{line_num}:{column_num}: {error_colored} {}",
        error_msg.to_lowercase()
    );

    let padded_previous_num = format!("{:5}", line_num - 1);
    let padded_num = format!("{:5}", line_num);

    if line_num != 1 {
        println!("{padded_previous_num} | {previous_error_line}");
    }
    println!("{padded_num} | {error_line}");
    println!("      |{}\x1b[1m\x1b[31m^", " ".repeat(column_num));
}
