use std::iter::Peekable;
use std::slice::Iter;

use crate::report;
use crate::token::Literal;
use crate::token_type::TokenType::*;
use crate::{expr::Expr, token::Token};

// parameters: token iterator, and a series of TokenType variants separated by |
// return option of next token
macro_rules! match_types {
    ($tokens:ident, $( $variant:pat_param )|* ) => {
        match $tokens.peek() {
            Some(token) => {
                match token.typ {
                    $(
                        $variant
                    )|* => $tokens.next(),
                    _ => None,
                }
            },
            None => None,
        }
    };
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, (Token, &'static str)> {
    let line_count = match tokens.last() {
        Some(token) => token.line,
        None => 0,
    };

    expression(line_count, &mut tokens.iter().peekable())
}

fn expression(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expr, (Token, &'static str)> {
    equality(line_count, tokens)
}

fn equality(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expr, (Token, &'static str)> {
    let mut expr = comparison(line_count, tokens);

    while let Some(operator) = match_types!(tokens, BangEqual | EqualEqual) {
        let operator = operator.to_owned();
        let right = comparison(line_count, tokens);
        expr = Ok(Expr::Binary {
            left: Box::new(expr?),
            operator,
            right: Box::new(right?),
        });
    }

    expr
}

fn comparison(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expr, (Token, &'static str)> {
    let mut expr = term(line_count, tokens)?;

    while let Some(operator) = match_types!(tokens, Greater | GreaterEqual | Less | LessEqual) {
        let operator = operator.to_owned();
        let right = term(line_count, tokens)?;
        expr = Expr::Binary {
            left: Box::new(expr),
            operator,
            right: Box::new(right),
        };
    }

    Ok(expr)
}

fn term(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expr, (Token, &'static str)> {
    let mut expr = factor(line_count, tokens);

    while let Some(operator) = match_types!(tokens, Minus | Plus) {
        let operator = operator.to_owned();
        let right = factor(line_count, tokens);
        expr = Ok(Expr::Binary {
            left: Box::new(expr?),
            operator,
            right: Box::new(right?),
        });
    }

    expr
}

fn factor(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expr, (Token, &'static str)> {
    let mut expr = unary(line_count, tokens);

    while let Some(operator) = match_types!(tokens, Slash | Star) {
        let operator = operator.to_owned();
        let right = unary(line_count, tokens);
        expr = Ok(Expr::Binary {
            left: Box::new(expr?),
            operator,
            right: Box::new(right?),
        });
    }

    expr
}

fn unary(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expr, (Token, &'static str)> {
    if let Some(operator) = match_types!(tokens, Bang | Minus) {
        let operator = operator.to_owned();
        let right = unary(line_count, tokens);
        Ok(Expr::Unary {
            operator,
            right: Box::new(right?),
        })
    } else {
        primary(line_count, tokens)
    }
}

fn primary(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
) -> Result<Expr, (Token, &'static str)> {
    match tokens.next() {
        Some(token) => match token.typ {
            False => Ok(Expr::LiteralExpr {
                value: Literal::BoolLiteral(false),
            }),
            True => Ok(Expr::LiteralExpr {
                value: Literal::BoolLiteral(true),
            }),
            Nil => Ok(Expr::LiteralExpr {
                value: Literal::None,
            }),
            Number | StringToken => Ok(Expr::LiteralExpr {
                value: token.literal.clone(),
            }),
            LeftParen => {
                let expr = expression(line_count, tokens);

                match tokens.next() {
                    Some(next_token) => match next_token.typ {
                        RightParen => Ok(Expr::Grouping {
                            expression: Box::new(expr?),
                        }),
                        _ => error(line_count, tokens, "Expected ')' after expression."),
                    },
                    None => error(
                        line_count,
                        tokens,
                        "Expected ')' after expression, instead found end of file.",
                    ),
                }
            }
            _ => Err((token.clone(), "Expected expression.")),
        },
        None => Err((
            generate_eof(line_count),
            "Expected expression, instead found end of file.",
        )),
    }
}

fn error(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    message: &'static str,
) -> Result<Expr, (Token, &'static str)> {
    match tokens.next() {
        Some(token) => {
            report(token.line, &format!(" at '{}'", token.lexeme), message);
            Err((token.clone(), message))
        }
        None => {
            report(line_count, " at end", message);
            Err((generate_eof(line_count), message))
        }
    }
}

fn synchronize(line_count: usize, tokens: &mut Peekable<Iter<Token>>) {
    while let Some(token) = tokens.next() {
        match token.typ {
            Semicolon => return,
            _ => {
                if let Some(token) = tokens.peek() {
                    match token.typ {
                        Class | Fun | Var | For | If | While | Print | Return => return,
                        _ => {}
                    }
                }
            }
        }
    }
}

fn generate_eof(line_count: usize) -> Token {
    Token {
        typ: Eof,
        lexeme: String::new(),
        literal: Literal::None,
        line: line_count,
    }
}
