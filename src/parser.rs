use std::iter::Peekable;
use std::slice::Iter;

use crate::expr::ExprKind;
use crate::report;
use crate::stmt::Stmt;
use crate::token::Literal;
use crate::token_type::TokenType::{self, *};
use crate::utils::{ExprId, Soo};
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

pub fn parse(tokens: Vec<Token>) -> Result<Vec<Stmt>, Vec<(Token, Soo)>> {
    let line_count = match tokens.last() {
        Some(token) => token.line,
        None => 0,
    };

    let mut id = ExprId::new();

    let token_iter = &mut tokens.iter().peekable();

    let mut statements = Vec::new();
    let mut errors = Vec::new();
    let mut had_error = false;
    while token_iter.peek().is_some() {
        match declaration(&mut id, line_count, token_iter, &mut had_error) {
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
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    let result = match tokens.peek().unwrap().typ {
        Fun => function("function", id, line_count, tokens, had_error),
        Var => var_declaration(id, line_count, tokens, had_error),
        _ => statement(id, line_count, tokens, had_error),
    };

    if result.is_err() {
        synchronize(line_count, tokens);
    }
    result
}

fn function(
    kind: &str,
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    tokens.next();

    let name = consume(
        Identifier,
        format!("Expected {kind} name, instead found end of file.").into(),
        format!("Expected {kind} name.").into(),
        line_count,
        tokens,
    )?
    .clone();
    consume(
        LeftParen,
        format!("Expected '(' after {kind} name, instead found end of file.").into(),
        format!("Expected '(' after {kind} name.").into(),
        line_count,
        tokens,
    )?;

    let mut parameters = Vec::new();
    if !check(RightParen, tokens) {
        loop {
            if parameters.len() >= 255 {
                report_error(
                    tokens,
                    line_count,
                    had_error,
                    "Can't have more than 255 parameters.".into(),
                );
            }

            parameters.push(
                consume(
                    Identifier,
                    "Expected parameter name.".into(),
                    "Expected parameter name, instead found end of file.".into(),
                    line_count,
                    tokens,
                )?
                .to_owned(),
            );

            if match_types!(tokens, Comma).is_none() {
                break;
            }
        }
    }

    consume(
        RightParen,
        "Expect ')' after paremeters.".into(),
        "Expect ')' after paremeters, instead found end of file.".into(),
        line_count,
        tokens,
    )?;

    if !check(LeftBrace, tokens) {
        consume(
            LeftBrace,
            format!("Expect '{{' before {kind} body.").into(),
            format!("Expect '{{' before {kind} body, instead found end of file.").into(),
            line_count,
            tokens,
        )?;
    }

    let body = block(id, line_count, tokens, had_error)?;
    Ok(Stmt::Function(crate::stmt::Function {
        name: name.to_owned(),
        params: parameters,
        body,
    }))
}

fn var_declaration(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    tokens.next();

    let stmt = match tokens.next() {
        Some(identifier) => match identifier.typ {
            Identifier => {
                let initializer = match tokens.peek() {
                    Some(next_token) => match next_token.typ {
                        Equal => {
                            tokens.next();
                            Some(Box::new(expression(id, line_count, tokens, had_error)?))
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
            _ => Err(error(line_count, tokens, "Expected variable name.".into())),
        },
        None => Err(error(
            line_count,
            tokens,
            "Expected variable name, instead found end of file.".into(),
        )),
    }?;

    match tokens.next() {
        Some(next_token) => match next_token.typ {
            Semicolon => Ok(stmt),
            _ => Err(error(
                line_count,
                tokens,
                "Expected ';' after variable declaration.".into(),
            )),
        },
        None => Err(error(
            line_count,
            tokens,
            "Expected ';' after variable declaration, instead found end of file.".into(),
        )),
    }
}

fn statement(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    match tokens.peek() {
        Some(next_token) => match next_token.typ {
            For => for_statement(id, line_count, tokens, had_error),
            If => if_statement(id, line_count, tokens, had_error),
            Print => print_statement(id, line_count, tokens, had_error),
            Return => return_statement(id, line_count, tokens, had_error),
            While => while_statement(id, line_count, tokens, had_error),
            LeftBrace => Ok(Stmt::Block {
                statements: block(id, line_count, tokens, had_error)?,
            }),
            _ => expression_statement(id, line_count, tokens, had_error),
        },
        None => Err(error(line_count, tokens, "Expected a statement.".into())),
    }
}

fn for_statement(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    tokens.next();

    consume(
        LeftParen,
        "Expected '(' after 'for', instead found end of file.".into(),
        "Expected '(' after 'for'.".into(),
        line_count,
        tokens,
    )?;

    let initializer = if let Some(_) = match_types!(tokens, Semicolon) {
        None
    } else if check(Var, tokens) {
        Some(var_declaration(id, line_count, tokens, had_error)?)
    } else {
        Some(expression_statement(id, line_count, tokens, had_error)?)
    };

    let condition = if !check(Semicolon, tokens) {
        Some(expression(id, line_count, tokens, had_error)?)
    } else {
        None
    };
    consume(
        Semicolon,
        "Expected ';' after loop condition, instead found end of file.".into(),
        "Expected ';' after loop condition.".into(),
        line_count,
        tokens,
    )?;

    let increment = if !check(RightParen, tokens) {
        Some(expression(id, line_count, tokens, had_error)?)
    } else {
        None
    };
    consume(
        RightParen,
        "Expected ')' after for clauses, instead found end of file.".into(),
        "Expected ')' after for clauses.".into(),
        line_count,
        tokens,
    )?;

    let mut body = statement(id, line_count, tokens, had_error)?;

    if let Some(increment_expr) = increment {
        body = Stmt::Block {
            statements: vec![
                body,
                Stmt::Expression {
                    expression: Box::new(increment_expr),
                },
            ],
        };
    }

    let condition = condition.unwrap_or(Expr(
        id.next(),
        ExprKind::LiteralExpr {
            value: Literal::BoolLiteral(false),
        },
    ));
    body = Stmt::While {
        condition: Box::new(condition),
        body: Box::new(body),
    };

    if let Some(initializer_stmt) = initializer {
        body = Stmt::Block {
            statements: vec![initializer_stmt, body],
        };
    }

    Ok(body)
}

fn if_statement(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    tokens.next();

    match tokens.next() {
        Some(left_paren) => match left_paren.typ {
            LeftParen => {
                let condition = expression(id, line_count, tokens, had_error)?;
                match tokens.next() {
                    Some(right_paren) => match right_paren.typ {
                        RightParen => {
                            let then_branch = statement(id, line_count, tokens, had_error)?;
                            let else_token = match_types!(tokens, Else);
                            let else_branch = match else_token {
                                Some(_) => Some(statement(id, line_count, tokens, had_error)?),
                                _ => None,
                            };

                            Ok(Stmt::If {
                                condition: Box::new(condition),
                                then_branch: Box::new(then_branch),
                                else_branch: else_branch
                                    .and_then(|else_stmt| Some(Box::new(else_stmt))),
                            })
                        }
                        _ => Err(error(
                            line_count,
                            tokens,
                            "Expected ')' after 'if' condition.".into(),
                        )),
                    },
                    _ => Err(error(
                        line_count,
                        tokens,
                        "Expected ')' after 'if' condition, instead found end of file.".into(),
                    )),
                }
            }
            _ => Err(error(line_count, tokens, "Expected '(' after 'if'.".into())),
        },
        _ => Err(error(
            line_count,
            tokens,
            "Expected '(' after 'if', instead found end of file.".into(),
        )),
    }
}

fn print_statement(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    tokens.next();
    let value = expression(id, line_count, tokens, had_error)?;

    match tokens.next() {
        Some(next_token) => match next_token.typ {
            Semicolon => Ok(Stmt::Print {
                expression: Box::new(value),
            }),
            _ => Err(error(
                line_count,
                tokens,
                "Expected ';' after value.".into(),
            )),
        },
        None => Err(error(
            line_count,
            tokens,
            "Expected ';' after value, instead found end of file.".into(),
        )),
    }
}

fn return_statement(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    let keyword = tokens.next().unwrap().to_owned();
    let value = if !check(Semicolon, tokens) {
        Some(Box::new(expression(id, line_count, tokens, had_error)?))
    } else {
        None
    };

    consume(
        Semicolon,
        "Expected ';' after return value, instead found end of file.".into(),
        "Expected ';' after return value.".into(),
        line_count,
        tokens,
    )?;
    Ok(Stmt::Return { keyword, value })
}

fn while_statement(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    tokens.next();

    match tokens.next() {
        Some(left_paren) => match left_paren.typ {
            LeftParen => {
                let condition = expression(id, line_count, tokens, had_error)?;
                match tokens.next() {
                    Some(right_paren) => match right_paren.typ {
                        RightParen => {
                            let body = statement(id, line_count, tokens, had_error)?;

                            Ok(Stmt::While {
                                condition: Box::new(condition),
                                body: Box::new(body),
                            })
                        }
                        _ => Err(error(
                            line_count,
                            tokens,
                            "Expected ')' after condition.".into(),
                        )),
                    },
                    _ => Err(error(
                        line_count,
                        tokens,
                        "Expected ')' after condition, instead found end of file.".into(),
                    )),
                }
            }
            _ => Err(error(
                line_count,
                tokens,
                "Expected '(' after 'while'.".into(),
            )),
        },
        _ => Err(error(
            line_count,
            tokens,
            "Expected '(' after 'while', instead found end of file.".into(),
        )),
    }
}

fn block(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Vec<Stmt>, (Token, Soo)> {
    tokens.next();
    let mut statements = Vec::new();

    while let Some(token) = tokens.peek() {
        match token.typ {
            RightBrace => {
                tokens.next();
                return Ok(statements);
            }
            _ => {
                let stmt = declaration(id, line_count, tokens, had_error)?;
                statements.push(stmt);
            }
        };
    }

    Err(error(
        line_count,
        tokens,
        "Expected '}' after block.".into(),
    ))
}

fn expression_statement(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Stmt, (Token, Soo)> {
    let expression = expression(id, line_count, tokens, had_error)?;

    match tokens.next() {
        Some(next_token) => match next_token.typ {
            Semicolon => Ok(Stmt::Expression {
                expression: Box::new(expression),
            }),
            _ => Err(error(
                line_count,
                tokens,
                "Expected ';' after expression.".into(),
            )),
        },
        None => Err(error(
            line_count,
            tokens,
            "Expected ';' after expression, instead found end of file.".into(),
        )),
    }
}

fn expression(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    assignment(id, line_count, tokens, had_error)
}

fn assignment(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let expr = or(id, line_count, tokens, had_error)?;

    match tokens.peek() {
        Some(token) => match token.typ {
            Equal => {
                tokens.next();
                let value = assignment(id, line_count, tokens, had_error)?;

                match expr.1 {
                    ExprKind::Variable { name } => Ok(Expr(
                        id.next(),
                        ExprKind::Assign {
                            name,
                            value: Box::new(value),
                        },
                    )),
                    _ => {
                        error(line_count, tokens, "Invalid assignment target.".into());
                        Ok(expr)
                    }
                }
            }
            _ => Ok(expr),
        },
        None => Ok(expr),
    }
}

fn or(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let mut expr = and(id, line_count, tokens, had_error)?;

    while let Some(operator) = match_types!(tokens, Or) {
        let operator = operator.to_owned();
        let right = and(id, line_count, tokens, had_error)?;
        expr = Expr(
            id.next(),
            ExprKind::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            },
        );
    }

    Ok(expr)
}

fn and(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let mut expr = equality(id, line_count, tokens, had_error)?;

    while let Some(operator) = match_types!(tokens, And) {
        let operator = operator.to_owned();
        let right = and(id, line_count, tokens, had_error)?;
        expr = Expr(
            id.next(),
            ExprKind::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            },
        );
    }

    Ok(expr)
}

fn equality(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let mut expr = comparison(id, line_count, tokens, had_error)?;

    while let Some(operator) = match_types!(tokens, BangEqual | EqualEqual) {
        let operator = operator.to_owned();
        let right = comparison(id, line_count, tokens, had_error)?;
        expr = Expr(
            id.next(),
            ExprKind::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            },
        );
    }

    Ok(expr)
}

fn comparison(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let mut expr = term(id, line_count, tokens, had_error)?;

    while let Some(operator) = match_types!(tokens, Greater | GreaterEqual | Less | LessEqual) {
        let operator = operator.to_owned();
        let right = term(id, line_count, tokens, had_error)?;
        expr = Expr(
            id.next(),
            ExprKind::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            },
        );
    }

    Ok(expr)
}

fn term(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let mut expr = factor(id, line_count, tokens, had_error);

    while let Some(operator) = match_types!(tokens, Minus | Plus) {
        let operator = operator.to_owned();
        let right = factor(id, line_count, tokens, had_error);
        expr = Ok(Expr(
            id.next(),
            ExprKind::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            },
        ));
    }

    expr
}

fn factor(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let mut expr = unary(id, line_count, tokens, had_error);

    while let Some(operator) = match_types!(tokens, Slash | Star) {
        let operator = operator.to_owned();
        let right = unary(id, line_count, tokens, had_error);
        expr = Ok(Expr(
            id.next(),
            ExprKind::Binary {
                left: Box::new(expr?),
                operator,
                right: Box::new(right?),
            },
        ));
    }

    expr
}

fn unary(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    if let Some(operator) = match_types!(tokens, Bang | Minus) {
        let operator = operator.to_owned();
        let right = unary(id, line_count, tokens, had_error);
        Ok(Expr(
            id.next(),
            ExprKind::Unary {
                operator,
                right: Box::new(right?),
            },
        ))
    } else {
        call(id, line_count, tokens, had_error)
    }
}

fn call(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let mut expr = primary(id, line_count, tokens, had_error)?;

    loop {
        if let Some(_) = match_types!(tokens, LeftParen) {
            expr = finish_call(id, expr, line_count, tokens, had_error)?;
        } else {
            break;
        }
    }

    Ok(expr)
}

fn finish_call(
    id: &mut ExprId,
    callee: Expr,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    let mut arguments = Vec::new();

    if !check(RightParen, tokens) {
        loop {
            if arguments.len() >= 255 {
                report_error(
                    tokens,
                    line_count,
                    had_error,
                    "Can't have more than 255 arguments.".into(),
                );
            }

            arguments.push(expression(id, line_count, tokens, had_error)?);
            if !match_types!(tokens, Comma).is_some() {
                break;
            }
        }
    }

    let paren = consume(
        RightParen,
        "Expected ')' after arguments, instead found end of file.".into(),
        "Expected ')' after arguments.".into(),
        line_count,
        tokens,
    )?
    .to_owned();

    Ok(Expr(
        id.next(),
        ExprKind::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        },
    ))
}

fn primary(
    id: &mut ExprId,
    line_count: usize,
    tokens: &mut Peekable<Iter<Token>>,
    had_error: &mut bool,
) -> Result<Expr, (Token, Soo)> {
    match tokens.next() {
        Some(token) => match token.typ {
            False => Ok(Expr(
                id.next(),
                ExprKind::LiteralExpr {
                    value: Literal::BoolLiteral(false),
                },
            )),
            True => Ok(Expr(
                id.next(),
                ExprKind::LiteralExpr {
                    value: Literal::BoolLiteral(true),
                },
            )),
            Nil => Ok(Expr(
                id.next(),
                ExprKind::LiteralExpr {
                    value: Literal::None,
                },
            )),
            Number | StringToken => Ok(Expr(
                id.next(),
                ExprKind::LiteralExpr {
                    value: token.literal.clone(),
                },
            )),
            Identifier => Ok(Expr(
                id.next(),
                ExprKind::Variable {
                    name: token.to_owned(),
                },
            )),
            LeftParen => {
                let expr = expression(id, line_count, tokens, had_error);

                match tokens.next() {
                    Some(next_token) => match next_token.typ {
                        RightParen => Ok(Expr(
                            id.next(),
                            ExprKind::Grouping {
                                expression: Box::new(expr?),
                            },
                        )),
                        _ => Err(error(
                            line_count,
                            tokens,
                            "Expected ')' after expression.".into(),
                        )),
                    },
                    None => Err(error(
                        line_count,
                        tokens,
                        "Expected ')' after expression, instead found end of file.".into(),
                    )),
                }
            }
            _ => Err((token.clone(), "Expected expression.".into())),
        },
        None => Err((
            generate_eof(line_count),
            "Expected expression, instead found end of file.".into(),
        )),
    }
}

fn consume<'t>(
    typ: TokenType,
    eof_message: Soo,
    message: Soo,
    line_count: usize,
    tokens: &'t mut Peekable<Iter<Token>>,
) -> Result<&'t Token, (Token, Soo)> {
    match tokens.next() {
        Some(token) => {
            if token.typ == typ {
                Ok(token)
            } else {
                Err(error(line_count, tokens, message))
            }
        }
        _ => Err(error(line_count, tokens, eof_message)),
    }
}

fn check(typ: TokenType, tokens: &mut Peekable<Iter<Token>>) -> bool {
    match tokens.peek() {
        Some(token) => typ == token.typ,
        _ => false,
    }
}

fn error(line_count: usize, tokens: &mut Peekable<Iter<Token>>, message: Soo) -> (Token, Soo) {
    match tokens.next() {
        Some(token) => {
            report(token.line, &format!(" at '{}'", token.lexeme), &message);
            (token.clone(), message)
        }
        None => {
            report(line_count, " at end", &message);
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

/// only report the error without throwing
fn report_error(
    tokens: &mut Peekable<Iter<Token>>,
    line_count: usize,
    had_error: &mut bool,
    message: Soo,
) {
    crate::error(
        tokens
            .peek()
            .and_then(|token| Some(token.line))
            .unwrap_or(line_count),
        &message,
    );
    *had_error = true;
}
