use std::iter::Peekable;
use std::slice::Iter;

use crate::report;
use crate::stmt::Stmt;
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

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, Vec<(Token, &'static str)>> {
    let line_count = match tokens.last() {
        Some(token) => token.line,
        None => 0,
    };

    let token_iter = &mut tokens.iter().peekable();

    let mut statements = Vec::new();
    let mut errors = Vec::new();
    let mut had_error = false;
    while token_iter.peek().is_some() {
        match declaration(line_count, token_iter, &mut had_error) {
            Ok(stmt) => statements.push(stmt),
            Err(error) => errors.push(error),
        };
    }

    if errors.is_empty() && !had_error {
        Ok(statements)
    } else {
        Err(errors)
    }
}

fn declaration(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, &'static str)> {
    let result = match tokens.peek().unwrap().typ {
        Var => var_declaration(line_count, tokens, had_error),
        _ => statement(line_count, tokens, had_error),
    };

    if result.is_err() {
        synchronize(line_count, tokens);
    }
    result
}

fn var_declaration(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, &'static str)> {
    tokens.next();

    let stmt = match tokens.next() {
        Some(identifier) => match identifier.typ {
            Identifier => {
                let initializer = match tokens.peek() {
                    Some(next_token) => match next_token.typ {
                        Equal => {
                            tokens.next();
                            Some(Box::new(expression(line_count, tokens, had_error)?))
                        }
                        _ => None,
                    },
                    _ => None,
                };
                Ok(Stmt::Var {
                    name: identifier.to_owned(),
                    initializer,
                })
            }
            _ => Err(error(line_count, tokens, "Expected variable name.")),
        },
        None => Err(error(
            line_count,
            tokens,
            "Expected variable name, instead found end of file.",
        )),
    }?;

    match tokens.next() {
        Some(next_token) => match next_token.typ {
            Semicolon => Ok(stmt),
            _ => Err(error(
                line_count,
                tokens,
                "Expected ';' after variable declaration.",
            )),
        },
        None => Err(error(
            line_count,
            tokens,
            "Expected ';' after variable declaration, instead found end of file.",
        )),
    }
}

fn statement(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, &'static str)> {
    match tokens.peek().unwrap().typ {
        Print => print_statement(line_count, tokens, had_error),
        LeftBrace => Ok(Stmt::Block {
            statements: block(line_count, tokens, had_error)?,
        }),
        _ => expression_statement(line_count, tokens, had_error),
    }
}

fn block(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Vec<Stmt>, (Token, &'static str)> {
    tokens.next();
    let mut statements = Vec::new();

    while let Some(token) = tokens.peek() {
        match token.typ {
            RightBrace => {
                tokens.next();
                return Ok(statements);
            }
            _ => {
                let stmt = declaration(line_count, tokens, had_error)?;
                statements.push(stmt);
            }
        };
    }

    Err(error(line_count, tokens, "Expected '}' after block."))
}

fn print_statement(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, &'static str)> {
    tokens.next();
    let value = expression(line_count, tokens, had_error)?;

    match tokens.next() {
        Some(next_token) => match next_token.typ {
            Semicolon => Ok(Stmt::Print {
                expression: Box::new(value),
            }),
            _ => Err(error(line_count, tokens, "Expected ';' after value.")),
        },
        None => Err(error(
            line_count,
            tokens,
            "Expected ';' after value, instead found end of file.",
        )),
    }
}

fn expression_statement(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, &'static str)> {
    let expression = expression(line_count, tokens, had_error)?;

    match tokens.next() {
        Some(next_token) => match next_token.typ {
            Semicolon => Ok(Stmt::Expression {
                expression: Box::new(expression),
            }),
            _ => Err(error(line_count, tokens, "Expected ';' after expression.")),
        },
        None => Err(error(
            line_count,
            tokens,
            "Expected ';' after expression, instead found end of file.",
        )),
    }
}

fn expression(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, &'static str)> {
    assignment(line_count, tokens, had_error)
}

fn assignment(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, &'static str)> {
    let expr = equality(line_count, tokens, had_error)?;

    match tokens.peek() {
        Some(token) => match token.typ {
            Equal => {
                tokens.next();
                let value = assignment(line_count, tokens, had_error)?;

                match expr {
                    Expr::Variable { name } => Ok(Expr::Assign {
                        name,
                        value: Box::new(value),
                    }),
                    _ => {
                        error(line_count, tokens, "Invalid assignment target.");
                        Ok(expr)
                    }
                }
            }
            _ => Ok(expr),
        },
        None => Ok(expr),
    }
}

fn equality(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, &'static str)> {
    let mut expr = comparison(line_count, tokens, had_error);

    while let Some(operator) = match_types!(tokens, BangEqual | EqualEqual) {
        let operator = operator.to_owned();
        let right = comparison(line_count, tokens, had_error);
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
    had_error: &mut bool,
) -> Result<Expr, (Token, &'static str)> {
    let mut expr = term(line_count, tokens, had_error)?;

    while let Some(operator) = match_types!(tokens, Greater | GreaterEqual | Less | LessEqual) {
        let operator = operator.to_owned();
        let right = term(line_count, tokens, had_error)?;
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
    had_error: &mut bool,
) -> Result<Expr, (Token, &'static str)> {
    let mut expr = factor(line_count, tokens, had_error);

    while let Some(operator) = match_types!(tokens, Minus | Plus) {
        let operator = operator.to_owned();
        let right = factor(line_count, tokens, had_error);
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
    had_error: &mut bool,
) -> Result<Expr, (Token, &'static str)> {
    let mut expr = unary(line_count, tokens, had_error);

    while let Some(operator) = match_types!(tokens, Slash | Star) {
        let operator = operator.to_owned();
        let right = unary(line_count, tokens, had_error);
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
    had_error: &mut bool,
) -> Result<Expr, (Token, &'static str)> {
    if let Some(operator) = match_types!(tokens, Bang | Minus) {
        let operator = operator.to_owned();
        let right = unary(line_count, tokens, had_error);
        Ok(Expr::Unary {
            operator,
            right: Box::new(right?),
        })
    } else {
        primary(line_count, tokens, had_error)
    }
}

fn primary(
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
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
            Identifier => Ok(Expr::Variable {
                name: token.to_owned(),
            }),
            LeftParen => {
                let expr = expression(line_count, tokens, had_error);

                match tokens.next() {
                    Some(next_token) => match next_token.typ {
                        RightParen => Ok(Expr::Grouping {
                            expression: Box::new(expr?),
                        }),
                        _ => Err(error(line_count, tokens, "Expected ')' after expression.")),
                    },
                    None => Err(error(
                        line_count,
                        tokens,
                        "Expected ')' after expression, instead found end of file.",
                    )),
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
) -> (Token, &'static str) {
    match tokens.next() {
        Some(token) => {
            report(token.line, &format!(" at '{}'", token.lexeme), message);
            (token.clone(), message)
        }
        None => {
            report(line_count, " at end", message);
            (generate_eof(line_count), message)
        }
    }
}

fn synchronize(_line_count: usize, tokens: &mut Peekable<Iter<Token>>) {
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
