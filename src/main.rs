use std::io::IsTerminal;
use std::{env, fs, process};

use holy_script::interpreter::Interpreter;
use holy_script::lexer::{Token, tokenize};
use holy_script::parser::{ParseError, Parser};
use holy_script::tree;

fn use_color() -> bool {
    std::io::stderr().is_terminal() && env::var_os("NO_COLOR").is_none()
}

fn red_bold(text: &str) -> String {
    if use_color() {
        format!("\x1b[1;31m{}\x1b[0m", text)
    } else {
        text.to_string()
    }
}

fn red(text: &str) -> String {
    if use_color() {
        format!("\x1b[31m{}\x1b[0m", text)
    } else {
        text.to_string()
    }
}

fn yellow(text: &str) -> String {
    if use_color() {
        format!("\x1b[33m{}\x1b[0m", text)
    } else {
        text.to_string()
    }
}

fn gray(text: &str) -> String {
    if use_color() {
        format!("\x1b[90m{}\x1b[0m", text)
    } else {
        text.to_string()
    }
}

fn bold(text: &str) -> String {
    if use_color() {
        format!("\x1b[1m{}\x1b[0m", text)
    } else {
        text.to_string()
    }
}

fn report_parse_error(source: &str, e: &ParseError) {
    eprintln!("\n{} - line {}, column {}:", red_bold("syntax error"), e.line, e.col);

    if e.line > 0 {
        if let Some(line_src) = source.lines().nth(e.line - 1) {
            eprintln!("  {} | {}", gray(&format!("{:>4}", e.line)), line_src);
            let arrow_pad = e.col.saturating_sub(1);
            eprintln!("       {}", red_bold(&format!("{}^", " ".repeat(arrow_pad))));
        }
    }

    eprintln!("  {}\n", yellow(&e.message));
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let show_tree = args.iter().any(|a| a == "--tree" || a == "-t");
    let file = args.iter().skip(1).find(|a| !a.starts_with('-')).unwrap_or_else(|| {
        eprintln!("Usage: holy [--tree] <file.holy>");
        process::exit(1);
    });

    let source = fs::read_to_string(file).unwrap_or_else(|e| {
        eprintln!("Error reading '{}': {}", file, e);
        process::exit(1);
    });

    // Lex
    let tokens = tokenize(&source);

    // Validate 'amen'
    let amens: Vec<_> = tokens.iter().filter(|s| s.token == Token::Amen).collect();
    match amens.len() {
        0 => {
            eprintln!(
                "\n{}: every holy program must end with {}\n",
                red_bold("error"),
                bold("amen")
            );
            process::exit(1);
        }
        1 => {} // ok
        n => {
            eprintln!(
                "\n{}: found {} 'amen' tokens - exactly one is required at the end:",
                red_bold("error"),
                n
            );
            for sp in &amens {
                if let Some(line_src) = source.lines().nth(sp.line - 1) {
                    eprintln!("  {} | {}", gray(&format!("{:>4}", sp.line)), line_src);
                    eprintln!("       {}", red_bold(&format!("{}^", " ".repeat(sp.col.saturating_sub(1)))));
                }
            }
            eprintln!();
            process::exit(1);
        }
    }

    // Parse
    let mut p = Parser::new(tokens);
    let program = p.parse_program().unwrap_or_else(|e| {
        report_parse_error(&source, &e);
        process::exit(1);
    });

    if show_tree {
        tree::print_program(&program);
        return;
    }

    // Interpret
    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(&program) {
        eprintln!("{}: {}", red("error"), e);
        process::exit(1);
    }
}
