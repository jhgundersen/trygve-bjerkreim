mod lexer;
mod parser;
mod interpreter;

use std::{env, fs, process};

const VERSION: &str = "0.1.0";

const HELP: &str = "\
tbv — Trygve Bjerkreim language interpreter
Trygve Bjerkrheim (1904-2001) i silikone

BRUK:
    tbv <fil.tb>        Køyr eit program
    tbv --repl          Interaktiv REPL
    tbv --version       Vis versjon
    tbv --help          Denne meldinga
";

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(String::as_str) {
        None | Some("--help") | Some("-h") => {
            print!("{}", HELP);
        }
        Some("--version") | Some("-v") => {
            println!("tbv {}", VERSION);
        }
        Some("--repl") => {
            repl();
        }
        Some(path) => {
            let source = fs::read_to_string(path).unwrap_or_else(|e| {
                eprintln!("tbv: kan ikkje lesa '{}': {}", path, e);
                process::exit(1);
            });
            run_source(&source, path);
        }
    }
}

fn run_source(source: &str, filename: &str) {
    let tokens = match lexer::tokenize(source) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Syntaksfeil i {}: {}", filename, e);
            process::exit(1);
        }
    };
    let program = match parser::parse(tokens) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Syntaksfeil i {}: {}", filename, e);
            process::exit(1);
        }
    };
    let mut interp = interpreter::Interpreter::new();
    if let Err(e) = interp.run(&program) {
        eprintln!("Feil: {}", e);
        process::exit(1);
    }
}

fn repl() {
    use std::io::{self, BufRead, Write};

    println!("tbv {} — Trygve Bjerkreim language interpreter", VERSION);
    println!("Trygve Bjerkrheim (1904-2001) i silikone");
    println!("Skriv 'avslutt' for å gå ut.\n");

    let stdin = io::stdin();
    let mut interp = interpreter::Interpreter::new();
    let mut buf = String::new();

    loop {
        let prompt = if buf.is_empty() { "🌅 " } else { "   " };
        print!("{}", prompt);
        io::stdout().flush().ok();

        let mut line = String::new();
        match stdin.lock().read_line(&mut line) {
            Ok(0) | Err(_) => break,
            Ok(_) => {}
        }

        let trimmed = line.trim_end_matches('\n').trim_end_matches('\r');

        if trimmed.trim() == "avslutt" {
            break;
        }

        buf.push_str(trimmed);
        buf.push('\n');

        // Try to parse; run on success, keep buffering on incomplete
        match lexer::tokenize(&buf) {
            Ok(tokens) => match parser::parse(tokens) {
                Ok(program) => {
                    if let Err(e) = interp.run(&program) {
                        eprintln!("Feil: {}", e);
                    }
                    buf.clear();
                }
                Err(_) => {
                    if trimmed.trim().is_empty() {
                        eprintln!("Syntaksfeil i inntasting.");
                        buf.clear();
                    }
                }
            },
            Err(_) => {
                if trimmed.trim().is_empty() {
                    eprintln!("Syntaksfeil i inntasting.");
                    buf.clear();
                }
            }
        }
    }
}
