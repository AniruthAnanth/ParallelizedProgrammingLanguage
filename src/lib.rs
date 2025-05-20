//! Parallelized Programming Language library

pub mod parser;
pub mod scanner;
pub mod compiler;
pub mod vm;

/// Parse a source string into an AST expression
pub fn parse_expr(source: &str) -> parser::Expr {
    let mut parser = parser::PrattParser::new(scanner::Scanner::new(source));
    parser.expr(0)
}

pub use parser::PrattParser;
pub use scanner::Scanner;
pub use compiler::{Compiler, BytecodeCompiler};
pub use vm::VM;

#[cfg(test)]
mod tests {
    use super::*;
    use scanner::Token;

    #[test]
    fn full_pipeline_basic() {
        let expr = parse_expr("7 * (8 + 9) - 3");
        let debug = format!("{:?}", expr);
        assert!(debug.contains("BinaryOp"));
        assert!(debug.contains("Number(7.0)"));
        assert!(debug.contains("Number(9.0)"));
    }

    #[test]
    fn full_pipeline_negative() {
        let expr = parse_expr("-1 + 5");
        let debug = format!("{:?}", expr);
        assert!(debug.contains("UnaryOp"));
        assert!(debug.contains("Number(1.0)"));
    }

    #[test]
    fn full_pipeline_multiple_ops() {
        let expr = parse_expr("1+2*3-4/2");
        let debug = format!("{:?}", expr);
        assert!(debug.contains("BinaryOp"));
        assert!(debug.contains("Star"));
        assert!(debug.contains("Slash"));
    }

    #[test]
    fn integration_parse_simple_expr() {
        let code = "10 - 4";
        let expr = parse_expr(code);
        let bytecode = compiler::BytecodeCompiler::compile(&expr);
        let result = vm::VM::run(bytecode);
        assert_eq!(result, 6.);
    }

    #[test]
    fn integration_parse_pipeline_in_main() {
        let code = "1 + 2 * (3 - 4)";
        let expr = parse_expr(code);
        let bytecode = compiler::BytecodeCompiler::compile(&expr);
        let result = vm::VM::run(bytecode);
        assert_eq!(result as i64, -1_i64);
    }

    #[test]
    fn integration_scan_sequence() {
        let code = "foo = 42; // comment \n spawn";
        let mut scanner = scanner::Scanner::new(code);
        let tokens: Vec<Token> = std::iter::from_fn(|| Some(scanner.next_token()))
            .take_while(|t| *t != Token::Eof)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("foo".into()),
                Token::Assign,
                Token::Number(42.),
                Token::Semicolon,
                Token::KeywordSpawn,
            ]
        );
    }

    #[test]
    fn integration_scan_all_tokens() {
        let code = "(1+2)*3 - jz 100;";
        let mut scanner = scanner::Scanner::new(code);
        let mut tokens = Vec::new();
        loop {
            let t = scanner.next_token();
            tokens.push(t.clone());
            if t == Token::Eof {
                break;
            }
        }
        assert_eq!(tokens.last(), Some(&Token::Eof));
        assert!(tokens.contains(&Token::LParen));
        assert!(tokens.contains(&Token::KeywordJz));
    }
}
