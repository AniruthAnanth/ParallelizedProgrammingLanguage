use clap::Parser;
use parallelized_programming_language::{parse_expr, BytecodeCompiler, VM};
use std::fs;
use std::io::{self, Write};

/// Parallelized Programming Language CLI
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the file to execute. If not provided, starts a REPL.
    file: Option<std::path::PathBuf>,
}

fn preprocess_code(code: &str, base_path: Option<&std::path::Path>) -> String {
    use std::collections::HashMap;
    let mut macros = HashMap::new();
    let mut output = String::new();
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#define ") {
            // #define MACRO value
            let rest = &trimmed[8..];
            if let Some((name, value)) = rest.split_once(' ') {
                macros.insert(name.to_string(), value.to_string());
            }
            continue;
        } else if trimmed.starts_with("#include ") {
            // #include "file"
            let rest = &trimmed[9..].trim();
            if let Some(include_path) = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
                let include_file = if let Some(base) = base_path {
                    base.parent().unwrap_or(base).join(include_path)
                } else {
                    std::path::PathBuf::from(include_path)
                };
                if let Ok(contents) = std::fs::read_to_string(&include_file) {
                    let included = preprocess_code(&contents, Some(&include_file));
                    output.push_str(&included);
                    output.push('\n');
                }
            }
            continue;
        }
        // Macro substitution
        let mut processed = line.to_string();
        for (k, v) in &macros {
            processed = processed.replace(k, v);
        }
        output.push_str(&processed);
        output.push('\n');
    }
    output
}

fn run_code_with_preprocessing(code: &str, base_path: Option<&std::path::Path>) {
    let preprocessed = preprocess_code(code, base_path);
    let expr = parse_expr(&preprocessed);
    let bytecode = BytecodeCompiler::compile(&expr);
    let _result = VM::run(bytecode);
}

fn main() {
    let cli = Cli::parse();
    if let Some(file_path) = cli.file {
        let code = fs::read_to_string(&file_path).expect("Failed to read file");
        run_code_with_preprocessing(&code, Some(&file_path));
    } else {
        println!("Parallelized Programming Language REPL. Type 'exit' to quit.");
        let stdin = io::stdin();
        loop {
            print!("> ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            if stdin.read_line(&mut input).is_err() { break; }
            let input = input.trim();
            if input == "exit" { break; }
            if !input.is_empty() {
                run_code_with_preprocessing(input, None);
            }
        }
    }
}
