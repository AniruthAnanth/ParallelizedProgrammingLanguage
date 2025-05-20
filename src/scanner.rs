#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Identifier(String),
    Number(f64),
    Plus,
    Minus,
    Star,
    Slash,
    Assign,
    Semicolon,
    LParen,
    RParen,
    KeywordSpawn,
    KeywordSync,
    KeywordBarrier,
    KeywordJump,
    KeywordJz,
    KeywordJnz,
    Eof,
}

pub struct Scanner<'a> {
    input: &'a str,
    pos: usize,
    current: Option<char>,
}

impl<'a> Scanner<'a> {
    pub fn new(input: &'a str) -> Self {
        let mut s = Scanner {
            input,
            pos: 0,
            current: None,
        };
        s.bump();
        s
    }
    fn bump(&mut self) {
        self.current = self.input[self.pos..].chars().next();
        if let Some(c) = self.current {
            self.pos += c.len_utf8();
        }
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            while matches!(self.current, Some(c) if c.is_whitespace()) {
                self.bump();
            }
            if self.current == Some('/') && self.peek() == Some('/') {
                while self.current != Some('\n') && self.current.is_some() {
                    self.bump();
                }
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn identifier_or_keyword(&mut self) -> Token {
        let mut ident = String::new();
        while let Some(c) = self.current {
            if c.is_alphanumeric() || c == '_' {
                ident.push(c);
                self.bump();
            } else {
                break;
            }
        }
        match ident.as_str() {
            "spawn" => Token::KeywordSpawn,
            "sync" => Token::KeywordSync,
            "barrier" => Token::KeywordBarrier,
            "jump" => Token::KeywordJump,
            "jz" => Token::KeywordJz,
            "jnz" => Token::KeywordJnz,
            _ => Token::Identifier(ident),
        }
    }

    fn number(&mut self) -> Token {
        let mut num_str = String::new();
        // Integer part
        while let Some(c) = self.current {
            if c.is_ascii_digit() {
                num_str.push(c);
                self.bump();
            } else {
                break;
            }
        }
        // Fractional part
        if self.current == Some('.') && self.peek().map_or(false, |c| c.is_ascii_digit()) {
            num_str.push('.');
            self.bump(); // consume '.'
            while let Some(c) = self.current {
                if c.is_ascii_digit() {
                    num_str.push(c);
                    self.bump();
                } else {
                    break;
                }
            }
        }
        
        let value = num_str.parse::<f64>().unwrap();
        Token::Number(value)
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        match self.current {
            Some(c) if c.is_ascii_alphabetic() || c == '_' => self.identifier_or_keyword(),
            Some(c) if c.is_ascii_digit() => self.number(),
            Some('+') => {
                self.bump();
                Token::Plus
            }
            Some('-') => {
                self.bump();
                Token::Minus
            }
            Some('*') => {
                self.bump();
                Token::Star
            }
            Some('/') => {
                self.bump();
                Token::Slash
            }
            Some('=') => {
                self.bump();
                Token::Assign
            }
            Some(';') => {
                self.bump();
                Token::Semicolon
            }
            Some('(') => {
                self.bump();
                Token::LParen
            }
            Some(')') => {
                self.bump();
                Token::RParen
            }
            None => Token::Eof,
            Some(c) => {
                panic!("Unexpected character: {}", c);
            }
        }
    }

    pub fn current_position(&self) -> usize {
        self.pos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_token() {
        let mut s = Scanner::new("123");
        assert_eq!(s.next_token(), Token::Number(123.));
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn test_float_token() {
        let mut s = Scanner::new("123.45");
        assert_eq!(s.next_token(), Token::Number(123.45));
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn test_identifier_token() {
        let mut s = Scanner::new("foo_bar");
        assert_eq!(s.next_token(), Token::Identifier("foo_bar".into()));
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn test_keywords() {
        let mut s = Scanner::new("spawn sync barrier jump jz jnz");
        assert_eq!(s.next_token(), Token::KeywordSpawn);
        assert_eq!(s.next_token(), Token::KeywordSync);
        assert_eq!(s.next_token(), Token::KeywordBarrier);
        assert_eq!(s.next_token(), Token::KeywordJump);
        assert_eq!(s.next_token(), Token::KeywordJz);
        assert_eq!(s.next_token(), Token::KeywordJnz);
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn test_operators_and_delimiters() {
        let mut s = Scanner::new("+-*/=;()");
        assert_eq!(s.next_token(), Token::Plus);
        assert_eq!(s.next_token(), Token::Minus);
        assert_eq!(s.next_token(), Token::Star);
        assert_eq!(s.next_token(), Token::Slash);
        assert_eq!(s.next_token(), Token::Assign);
        assert_eq!(s.next_token(), Token::Semicolon);
        assert_eq!(s.next_token(), Token::LParen);
        assert_eq!(s.next_token(), Token::RParen);
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    fn test_whitespace_and_comments() {
        let code = "  42  // comment line\n +7\t";
        let mut s = Scanner::new(code);
        assert_eq!(s.next_token(), Token::Number(42.));
        assert_eq!(s.next_token(), Token::Plus);
        assert_eq!(s.next_token(), Token::Number(7.));
        assert_eq!(s.next_token(), Token::Eof);
    }

    #[test]
    #[should_panic]
    fn test_unexpected_character() {
        let mut s = Scanner::new("@");
        let _ = s.next_token();
    }
}
