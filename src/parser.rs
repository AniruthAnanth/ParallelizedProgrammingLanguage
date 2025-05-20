#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    Ident(String),
    UnaryOp {
        op: Token,
        rhs: Box<Expr>,
    },
    BinaryOp {
        lhs: Box<Expr>,
        op: Token,
        rhs: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Expr>,
    },
}

use crate::scanner::{Scanner, Token};

/// A Pratt parser for arithmetic expressions.
pub struct PrattParser<'a> {
    scanner: Scanner<'a>,
    current: Token,
}

impl<'a> PrattParser<'a> {
    pub fn new(scanner: Scanner<'a>) -> Self {
        let mut parser = PrattParser {
            scanner,
            current: Token::Eof,
        };
        parser.advance();
        parser
    }

    fn advance(&mut self) {
        self.current = self.scanner.next_token();
    }

    pub fn parse_function(&mut self) -> Expr {
        // Expect 'fn'
        self.advance();
        let name = if let Token::Identifier(name) = &self.current {
            name.clone()
        } else {
            panic!("Expected function name after 'fn'");
        };
        self.advance();
        // Parse parameters
        if self.current != Token::LParen {
            panic!("Expected '(' after function name");
        }
        self.advance();
        let mut params = Vec::new();
        while let Token::Identifier(param) = &self.current {
            params.push(param.clone());
            self.advance();
            if self.current == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        if self.current != Token::RParen {
            panic!("Expected ')' after parameters");
        }
        self.advance();
        // Parse body
        if self.current != Token::LBrace {
            panic!("Expected '{{' to start function body");
        }
        self.advance();
        let mut body = Vec::new();
        while self.current != Token::RBrace && self.current != Token::Eof {
            body.push(self.expr(0));
            if self.current == Token::Semicolon {
                self.advance();
            }
        }
        if self.current != Token::RBrace {
            panic!("Expected '}}' to end function body");
        }
        self.advance();
        Expr::Function { name, params, body }
    }

    pub fn parse_call(&mut self, name: String) -> Expr {
        // Already saw identifier and '('
        self.advance();
        let mut args = Vec::new();
        while self.current != Token::RParen && self.current != Token::Eof {
            args.push(self.expr(0));
            if self.current == Token::Comma {
                self.advance();
            } else {
                break;
            }
        }
        if self.current != Token::RParen {
            panic!("Expected ')' after arguments");
        }
        self.advance();
        Expr::Call { name, args }
    }

    fn nud(&mut self) -> Expr {
        match &self.current {
            Token::Number(n) => {
                let n = *n;
                self.advance();
                Expr::Number(n)
            }
            Token::Identifier(name) => {
                let name = name.clone();
                self.advance();
                if self.current == Token::LParen {
                    self.parse_call(name)
                } else {
                    Expr::Ident(name)
                }
            }
            Token::Minus => {
                self.advance();
                Expr::UnaryOp {
                    op: Token::Minus,
                    rhs: Box::new(self.expr(100)),
                }
            }
            Token::LParen => {
                self.advance();
                let expr = self.expr(0);
                if self.current != Token::RParen {
                    panic!(
                        "+Expected ')' but found {:?} at position {}",
                        self.current,
                        self.scanner.current_position()
                    );
                }
                self.advance();
                expr
            }
            Token::KeywordFn => self.parse_function(),
            _ => panic!("Unexpected token in nud: {:?}", self.current),
        }
    }

    fn lbp(token: &Token) -> u8 {
        match token {
            Token::Plus | Token::Minus => 10,
            Token::Star | Token::Slash => 20,
            _ => 0,
        }
    }

    fn led(&mut self, lhs: Expr, token: Token) -> Expr {
        match token {
            Token::Plus | Token::Minus | Token::Star | Token::Slash => {
                let op = token;
                let rbp = Self::lbp(&op);
                let rhs = self.expr(rbp);
                Expr::BinaryOp {
                    lhs: Box::new(lhs),
                    op,
                    rhs: Box::new(rhs),
                }
            }
            Token::RParen | Token::Eof => lhs,
            _ => panic!("Unexpected token in led: {:?}", token),
        }
    }

    pub fn expr(&mut self, min_bp: u8) -> Expr {
        let mut lhs = self.nud();
        loop {
            if self.current == Token::Eof || self.current == Token::RParen {
                break;
            }
            let lbp = Self::lbp(&self.current);
            if lbp < min_bp {
                break;
            }
            let op = self.current.clone();
            self.advance();
            lhs = self.led(lhs, op);
        }
        lhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::Scanner;
    use crate::scanner::Token;

    fn parse(code: &str) -> Expr {
        let mut parser = PrattParser::new(Scanner::new(code));
        parser.expr(0)
    }

    #[test]
    fn test_parse_number() {
        assert_eq!(parse("42"), Expr::Number(42.));
    }

    #[test]
    fn test_parse_identifier() {
        assert_eq!(parse("foo"), Expr::Ident("foo".into()));
    }

    #[test]
    fn test_parse_add() {
        let expr = parse("1+2");
        assert_eq!(
            expr,
            Expr::BinaryOp {
                lhs: Box::new(Expr::Number(1.)),
                op: Token::Plus,
                rhs: Box::new(Expr::Number(2.)),
            }
        );
    }

    #[test]
    fn test_parse_precedence() {
        let expr = parse("1+2*3");
        assert_eq!(
            expr,
            Expr::BinaryOp {
                lhs: Box::new(Expr::Number(1.)),
                op: Token::Plus,
                rhs: Box::new(Expr::BinaryOp {
                    lhs: Box::new(Expr::Number(2.)),
                    op: Token::Star,
                    rhs: Box::new(Expr::Number(3.)),
                }),
            }
        );
    }
    #[test]
    fn test_parse_parentheses() {
        let input = "(1+2)*3";
        let expr = parse(input);
        assert_eq!(
            expr,
            Expr::BinaryOp {
                lhs: Box::new(Expr::BinaryOp {
                    lhs: Box::new(Expr::Number(1.)),
                    op: Token::Plus,
                    rhs: Box::new(Expr::Number(2.)),
                }),
                op: Token::Star,
                rhs: Box::new(Expr::Number(3.)),
            }
        );
    }
    #[test]
    fn test_parse_unary() {
        let expr = parse("-5+2");
        assert_eq!(
            expr,
            Expr::BinaryOp {
                lhs: Box::new(Expr::UnaryOp {
                    op: Token::Minus,
                    rhs: Box::new(Expr::Number(5.)),
                }),
                op: Token::Plus,
                rhs: Box::new(Expr::Number(2.)),
            }
        );
    }
}
