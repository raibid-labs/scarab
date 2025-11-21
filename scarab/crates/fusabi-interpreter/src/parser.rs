use nom::{
    branch::alt,
    bytes::complete::{tag, take_until, take_while, take_while1},
    character::complete::{alpha1, alphanumeric1, char, digit1, multispace0, multispace1, one_of},
    combinator::{map, map_res, opt, recognize, value},
    multi::{many0, many1, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use nom_locate::LocatedSpan;

use crate::ast::*;
use crate::error::{FusabiError, Result};

pub type Span<'a> = LocatedSpan<&'a str>;

/// Parse a complete module/script
pub fn parse_module(input: &str) -> Result<Module> {
    let span = Span::new(input);
    match module(span) {
        Ok((_, module)) => Ok(module),
        Err(nom::Err::Error(e)) | Err(nom::Err::Failure(e)) => {
            let line = e.input.location_line();
            let col = e.input.get_column();
            Err(FusabiError::parse_error(
                line,
                col as u32,
                format!("Failed to parse: {:?}", e),
            ))
        }
        Err(nom::Err::Incomplete(_)) => Err(FusabiError::parse_error(0, 0, "Incomplete input")),
    }
}

// Whitespace and comments
fn ws<'a, F, O>(inner: F) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, O>
where
    F: FnMut(Span<'a>) -> IResult<Span<'a>, O>,
{
    delimited(multispace0, inner, multispace0)
}

fn comment(input: Span) -> IResult<Span, ()> {
    value(
        (),
        pair(tag("//"), take_while(|c| c != '\n')),
    )(input)
}

fn ws_or_comment(input: Span) -> IResult<Span, ()> {
    value(
        (),
        many0(alt((value((), multispace1), comment))),
    )(input)
}

// Identifiers
fn identifier(input: Span) -> IResult<Span, String> {
    map(
        recognize(pair(
            alt((alpha1, tag("_"))),
            many0(alt((alphanumeric1, tag("_")))),
        )),
        |s: Span| s.fragment().to_string(),
    )(input)
}

// Literals
fn int_literal(input: Span) -> IResult<Span, Value> {
    map_res(
        recognize(pair(opt(char('-')), digit1)),
        |s: Span| s.fragment().parse::<i64>().map(Value::Int),
    )(input)
}

fn float_literal(input: Span) -> IResult<Span, Value> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            digit1,
            char('.'),
            digit1,
        ))),
        |s: Span| s.fragment().parse::<f64>().map(Value::Float),
    )(input)
}

fn bool_literal(input: Span) -> IResult<Span, Value> {
    alt((
        value(Value::Bool(true), tag("true")),
        value(Value::Bool(false), tag("false")),
    ))(input)
}

fn string_literal(input: Span) -> IResult<Span, Value> {
    delimited(
        char('"'),
        map(
            take_while(|c| c != '"'),
            |s: Span| Value::String(s.fragment().to_string()),
        ),
        char('"'),
    )(input)
}

fn nil_literal(input: Span) -> IResult<Span, Value> {
    value(Value::Nil, tag("nil"))(input)
}

fn literal(input: Span) -> IResult<Span, Expr> {
    map(
        alt((
            float_literal,
            int_literal,
            bool_literal,
            string_literal,
            nil_literal,
        )),
        Expr::Literal,
    )(input)
}

// List literal
fn list_literal(input: Span) -> IResult<Span, Expr> {
    map(
        delimited(
            ws(char('[')),
            separated_list0(ws(char(',')), expr),
            ws(char(']')),
        ),
        Expr::ListLiteral,
    )(input)
}

// Map literal
fn map_entry(input: Span) -> IResult<Span, (String, Expr)> {
    let (input, key) = ws(identifier)(input)?;
    let (input, _) = ws(char('='))(input)?;
    let (input, value) = expr(input)?;
    Ok((input, (key, value)))
}

fn map_literal(input: Span) -> IResult<Span, Expr> {
    map(
        delimited(
            ws(char('{')),
            separated_list0(ws(char(';')), map_entry),
            ws(char('}')),
        ),
        Expr::MapLiteral,
    )(input)
}

// Variable
fn variable(input: Span) -> IResult<Span, Expr> {
    map(identifier, Expr::Variable)(input)
}

// Lambda: fun x y -> expr
fn lambda(input: Span) -> IResult<Span, Expr> {
    let (input, _) = ws(tag("fun"))(input)?;
    let (input, params) = many1(ws(identifier))(input)?;
    let (input, _) = ws(tag("->"))(input)?;
    let (input, body) = expr(input)?;
    Ok((
        input,
        Expr::Lambda {
            params,
            body: Box::new(body),
        },
    ))
}

// Let binding: let x = expr in body
fn let_expr(input: Span) -> IResult<Span, Expr> {
    let (input, _) = ws(tag("let"))(input)?;
    let (input, name) = ws(identifier)(input)?;
    let (input, _) = ws(char('='))(input)?;
    let (input, value) = ws(expr)(input)?;
    let (input, _) = ws(tag("in"))(input)?;
    let (input, body) = expr(input)?;
    Ok((
        input,
        Expr::Let {
            name,
            value: Box::new(value),
            body: Box::new(body),
        },
    ))
}

// If expression: if cond then expr else expr
fn if_expr(input: Span) -> IResult<Span, Expr> {
    let (input, _) = ws(tag("if"))(input)?;
    let (input, condition) = ws(expr)(input)?;
    let (input, _) = ws(tag("then"))(input)?;
    let (input, then_expr) = ws(expr)(input)?;
    let (input, else_expr) = opt(preceded(ws(tag("else")), expr))(input)?;
    Ok((
        input,
        Expr::If {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: else_expr.map(Box::new),
        },
    ))
}

// Parenthesized expression
fn paren_expr(input: Span) -> IResult<Span, Expr> {
    delimited(ws(char('(')), expr, ws(char(')')))(input)
}

// Primary expressions (atoms)
fn primary(input: Span) -> IResult<Span, Expr> {
    alt((
        literal,
        lambda,
        let_expr,
        if_expr,
        list_literal,
        map_literal,
        paren_expr,
        variable,
    ))(input)
}

// Postfix operations (function call, indexing, field access)
fn postfix(input: Span) -> IResult<Span, Expr> {
    let (input, mut result_expr) = primary(input)?;

    let mut remaining = input;
    loop {
        // Try function call
        if let Ok((input2, args)) = delimited(
            ws(char('(')),
            separated_list0(ws(char(',')), |i| expr(i)),
            ws(char(')')),
        )(remaining)
        {
            result_expr = Expr::Call {
                func: Box::new(result_expr),
                args,
            };
            remaining = input2;
            continue;
        }

        // Try indexing
        if let Ok((input2, index)) = delimited(ws(char('[')), |i| expr(i), ws(char(']')))(remaining) {
            result_expr = Expr::Index {
                expr: Box::new(result_expr),
                index: Box::new(index),
            };
            remaining = input2;
            continue;
        }

        // Try field access
        if let Ok((input2, field)) = preceded(ws(char('.')), identifier)(remaining) {
            result_expr = Expr::Field {
                expr: Box::new(result_expr),
                field,
            };
            remaining = input2;
            continue;
        }

        break;
    }

    Ok((remaining, result_expr))
}

// Unary operations
fn unary(input: Span) -> IResult<Span, Expr> {
    alt((
        map(
            pair(ws(char('-')), unary),
            |(_, e)| Expr::UnaryOp {
                op: UnaryOp::Neg,
                expr: Box::new(e),
            },
        ),
        map(
            pair(ws(char('!')), unary),
            |(_, e)| Expr::UnaryOp {
                op: UnaryOp::Not,
                expr: Box::new(e),
            },
        ),
        postfix,
    ))(input)
}

// Binary operations (with precedence)
fn binary_op(op_chars: &str, op: BinOp) -> impl Fn(Span) -> IResult<Span, BinOp> + '_ {
    move |input| value(op, ws(tag(op_chars)))(input)
}

fn multiplicative(input: Span) -> IResult<Span, Expr> {
    let (input, left) = unary(input)?;
    let (input, ops_and_exprs) = many0(pair(
        alt((
            binary_op("*", BinOp::Mul),
            binary_op("/", BinOp::Div),
            binary_op("%", BinOp::Mod),
        )),
        unary,
    ))(input)?;

    Ok((
        input,
        ops_and_exprs
            .into_iter()
            .fold(left, |acc, (op, right)| Expr::BinOp {
                op,
                left: Box::new(acc),
                right: Box::new(right),
            }),
    ))
}

fn additive(input: Span) -> IResult<Span, Expr> {
    let (input, left) = multiplicative(input)?;
    let (input, ops_and_exprs) = many0(pair(
        alt((
            binary_op("+", BinOp::Add),
            binary_op("-", BinOp::Sub),
        )),
        multiplicative,
    ))(input)?;

    Ok((
        input,
        ops_and_exprs
            .into_iter()
            .fold(left, |acc, (op, right)| Expr::BinOp {
                op,
                left: Box::new(acc),
                right: Box::new(right),
            }),
    ))
}

fn comparison(input: Span) -> IResult<Span, Expr> {
    let (input, left) = additive(input)?;
    let (input, op_and_expr) = opt(pair(
        alt((
            binary_op("==", BinOp::Eq),
            binary_op("!=", BinOp::Ne),
            binary_op("<=", BinOp::Le),
            binary_op("<", BinOp::Lt),
            binary_op(">=", BinOp::Ge),
            binary_op(">", BinOp::Gt),
        )),
        additive,
    ))(input)?;

    Ok((
        input,
        if let Some((op, right)) = op_and_expr {
            Expr::BinOp {
                op,
                left: Box::new(left),
                right: Box::new(right),
            }
        } else {
            left
        },
    ))
}

fn logical_and(input: Span) -> IResult<Span, Expr> {
    let (input, left) = comparison(input)?;
    let (input, rights) = many0(preceded(binary_op("&&", BinOp::And), comparison))(input)?;

    Ok((
        input,
        rights.into_iter().fold(left, |acc, right| Expr::BinOp {
            op: BinOp::And,
            left: Box::new(acc),
            right: Box::new(right),
        }),
    ))
}

fn logical_or(input: Span) -> IResult<Span, Expr> {
    let (input, left) = logical_and(input)?;
    let (input, rights) = many0(preceded(binary_op("||", BinOp::Or), logical_and))(input)?;

    Ok((
        input,
        rights.into_iter().fold(left, |acc, right| Expr::BinOp {
            op: BinOp::Or,
            left: Box::new(acc),
            right: Box::new(right),
        }),
    ))
}

// Top-level expression
fn expr(input: Span) -> IResult<Span, Expr> {
    logical_or(input)
}

// Statements
fn let_statement(input: Span) -> IResult<Span, Statement> {
    let (input, _) = ws(tag("let"))(input)?;
    let (input, name) = ws(identifier)(input)?;
    let (input, _) = ws(char('='))(input)?;
    let (input, value) = ws(expr)(input)?;
    Ok((input, Statement::Let { name, value }))
}

fn function_statement(input: Span) -> IResult<Span, Statement> {
    let (input, _) = ws(tag("let"))(input)?;
    let (input, name) = ws(identifier)(input)?;
    let (input, params) = many1(ws(identifier))(input)?;
    let (input, _) = ws(char('='))(input)?;
    let (input, body) = ws(expr)(input)?;
    Ok((
        input,
        Statement::Function {
            name,
            params,
            body: Box::new(body),
        },
    ))
}

fn expr_statement(input: Span) -> IResult<Span, Statement> {
    map(expr, Statement::Expr)(input)
}

fn statement(input: Span) -> IResult<Span, Statement> {
    ws_or_comment(input)?;
    let result = alt((function_statement, let_statement, expr_statement))(input);
    result
}

// Module (top-level)
fn module(input: Span) -> IResult<Span, Module> {
    let (input, _) = ws_or_comment(input)?;
    let (input, statements) = many0(terminated(statement, ws_or_comment))(input)?;
    let (input, _) = ws_or_comment(input)?;
    Ok((input, Module::new(statements)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_int() {
        let result = parse_module("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_string() {
        let result = parse_module(r#""hello""#);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_let() {
        let result = parse_module("let x = 42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_function() {
        let result = parse_module("let add x y = x + y");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if() {
        let result = parse_module("if true then 1 else 2");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_lambda() {
        let result = parse_module("fun x -> x + 1");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_list() {
        let result = parse_module("[1, 2, 3]");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_map() {
        let result = parse_module("{ x = 1; y = 2 }");
        assert!(result.is_ok());
    }
}
