use lazy_static::lazy_static;
use std::collections::HashMap;
use std::iter::Peekable;
use std::mem;
use std::str::Chars;

use crate::error;
use crate::token::{Literal, Token};
use crate::token_type::TokenType::{self, *};

lazy_static! {
    static ref KEYWORDS: HashMap<&'static str, TokenType> = HashMap::from([
        ("and", And),
        ("class", Class),
        ("else", Else),
        ("false", False),
        ("for", For),
        ("fun", Fun),
        ("if", If),
        ("nil", Nil),
        ("or", Or),
        ("print", Print),
        ("return", Return),
        ("super", Super),
        ("this", This),
        ("true", True),
        ("var", Var),
        ("while", While),
    ]);
}

pub struct Scanner<'a> {
    source: Peekable<Chars<'a>>,
    tokens: Vec<Token>,
    text: String,
    line: usize,
}

impl<'a> Scanner<'a> {
    pub fn new(source: &'a str) -> Self {
        Scanner {
            source: source.chars().peekable(),
            tokens: Vec::new(),
            text: String::new(),
            line: 1,
        }
    }

    pub fn scan_tokens(mut self) -> (Vec<Token>, bool) {
        let mut had_error = false;

        while let Some(c) = self.get_next_token() {
            self.text.push(c);
            had_error |= self.scan_token(c);
        }

        // self.tokens.push(Token {
        //     typ: Eof,
        //     lexeme: String::new(),
        //     literal: Literal::None,
        //     line: self.line,
        // });
        (self.tokens, had_error)
    }

    fn get_next_token(&mut self) -> Option<char> {
        self.source.next()
    }

    fn scan_token(&mut self, c: char) -> bool {
        match c {
            '(' => self.add_token(LeftParen),
            ')' => self.add_token(RightParen),
            '{' => self.add_token(LeftBrace),
            '}' => self.add_token(RightBrace),
            ',' => self.add_token(Comma),
            '.' => self.add_token(Dot),
            '-' => self.add_token(Minus),
            '+' => self.add_token(Plus),
            ';' => self.add_token(Semicolon),
            '*' => self.add_token(Star),
            '!' => {
                let matched = self.match_next('=');
                self.add_token(if matched { BangEqual } else { Bang })
            }
            '=' => {
                let matched = self.match_next('=');
                self.add_token(if matched { EqualEqual } else { Equal })
            }
            '<' => {
                let matched = self.match_next('=');
                self.add_token(if matched { LessEqual } else { Less })
            }
            '>' => {
                let matched = self.match_next('=');
                self.add_token(if matched { GreaterEqual } else { Greater })
            }
            '/' => {
                if self.match_next('/') {
                    while let Some(&char) = self.source.peek() {
                        if char == '\n' {
                            break;
                        }
                        self.source.next();
                        self.text.clear();
                    }
                } else {
                    self.add_token(Slash);
                }
            }
            ' ' | '\r' | '\t' => {
                self.text.pop();
            }
            '\n' => {
                self.line += 1;
                self.text.pop();
            }
            '"' => self.scan_string(),
            _ => {
                if self.is_digit(c) {
                    self.scan_number();
                } else if self.is_alpha(c) {
                    self.scan_identifier();
                } else {
                    error(self.line, &("Unexpected character.".into()));
                    self.text.pop();
                    return true;
                }
            }
        };
        false
    }

    fn scan_string(&mut self) {
        while let Some(&c) = self.source.peek() {
            if c == '"' {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            self.text.push(c);
            self.source.next();
        }

        if self.source.peek().is_none() {
            error(self.line, &("Unterminated string.".into()));
            return;
        }

        // closing "
        self.source.next();

        self.text.remove(0);
        self.add_token(StringToken);
    }

    fn is_digit(&self, c: char) -> bool {
        '0' <= c && c <= '9'
    }

    fn scan_number(&mut self) {
        self.advance_digits();

        // check for a fractional part
        if let Some(&c) = self.source.peek() {
            if c == '.' {
                // clone the source iterator so that we can peek 2 characters ahead
                let mut cloned = self.source.clone();
                cloned.next();
                if let Some(&next_c) = cloned.peek() {
                    if self.is_digit(next_c) {
                        self.text.push(c);
                        self.source.next();
                        self.advance_digits();
                    }
                }
            }
        }

        self.add_token(Number);
    }

    fn advance_digits(&mut self) {
        while let Some(&c) = self.source.peek() {
            if !self.is_digit(c) {
                break;
            }
            self.text.push(c);
            self.source.next();
        }
    }

    fn is_alpha(&self, c: char) -> bool {
        ('a' <= c && c <= 'z') || ('A' <= c && c <= 'Z') || c == '_'
    }

    fn is_alpha_num(&self, c: char) -> bool {
        self.is_alpha(c) || self.is_digit(c)
    }

    fn scan_identifier(&mut self) {
        while let Some(&c) = self.source.peek() {
            if !self.is_alpha_num(c) {
                break;
            }
            self.text.push(c);
            self.source.next();
        }

        let typ = *KEYWORDS.get(&self.text as &str).unwrap_or(&Identifier);

        self.add_token(typ);
    }

    fn match_next(&mut self, expected: char) -> bool {
        if let Some(&next_char) = self.source.peek() {
            if next_char != expected {
                return false;
            }
        } else {
            return false;
        }

        self.source.next();
        self.text.push(expected);
        true
    }

    fn add_token(&mut self, typ: TokenType) {
        let mut lexeme = String::new();
        mem::swap(&mut self.text, &mut lexeme);

        // parse literals
        let literal: Literal = match typ {
            Identifier => Literal::IdentifierLiteral(lexeme.clone()),
            StringToken => Literal::StringLiteral(lexeme.clone()),
            Number => Literal::F64(lexeme.parse().unwrap()),
            _ => Literal::None,
        };

        self.tokens.push(Token {
            typ,
            lexeme,
            literal,
            line: self.line,
        });
    }
}
