use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::ast::{BinOp, Block, Expr, Program, Stmt};

#[derive(Parser)]
#[grammar = "lipona.pest"]
pub struct LiponaParser;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    Pest(#[from] pest::error::Error<Rule>),
    #[error("Unexpected rule: {0:?}")]
    UnexpectedRule(Rule),
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
    #[error("Parse error: missing inner element in {0:?}")]
    MissingInner(Rule),
}

pub fn parse(input: &str) -> Result<Program, ParseError> {
    let pairs = LiponaParser::parse(Rule::program, input)?;
    let mut stmts = Vec::new();

    for pair in pairs {
        if pair.as_rule() == Rule::program {
            for inner in pair.into_inner() {
                if inner.as_rule() == Rule::stmt {
                    stmts.push(parse_stmt(inner)?);
                }
            }
        }
    }

    Ok(stmts)
}

fn parse_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let inner = pair.into_inner().next().ok_or(ParseError::MissingInner(Rule::stmt))?;

    match inner.as_rule() {
        Rule::func_def => parse_func_def(inner),
        Rule::if_stmt => parse_if_stmt(inner),
        Rule::while_stmt => parse_while_stmt(inner),
        Rule::return_stmt => parse_return_stmt(inner),
        Rule::assign_stmt => parse_assign_stmt(inner),
        Rule::expr_stmt => {
            let expr = parse_expr(inner.into_inner().next().ok_or(ParseError::MissingInner(Rule::expr_stmt))?)?;
            Ok(Stmt::Expr(expr))
        }
        rule => Err(ParseError::UnexpectedRule(rule)),
    }
}

fn parse_func_def(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().ok_or(ParseError::MissingInner(Rule::func_def))?.as_str().to_string();

    let mut params = Vec::new();
    let mut body = Vec::new();

    for item in inner {
        match item.as_rule() {
            Rule::param_list => {
                for param in item.into_inner() {
                    params.push(param.as_str().to_string());
                }
            }
            Rule::stmt => {
                body.push(parse_stmt(item)?);
            }
            _ => {}
        }
    }

    Ok(Stmt::FuncDef { name, params, body })
}

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().ok_or(ParseError::MissingInner(Rule::if_stmt))?)?;

    let mut then_block: Block = Vec::new();
    let mut else_block: Option<Block> = None;
    let mut in_else = false;

    for item in inner {
        match item.as_rule() {
            Rule::stmt => {
                if in_else {
                    else_block.get_or_insert_with(Vec::new).push(parse_stmt(item)?);
                } else {
                    then_block.push(parse_stmt(item)?);
                }
            }
            _ => {
                // After first block of statements, we're in else
                if !then_block.is_empty() {
                    in_else = true;
                }
            }
        }
    }

    Ok(Stmt::If {
        cond,
        then_block,
        else_block,
    })
}

fn parse_while_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().ok_or(ParseError::MissingInner(Rule::while_stmt))?)?;

    let mut body = Vec::new();
    for item in inner {
        if item.as_rule() == Rule::stmt {
            body.push(parse_stmt(item)?);
        }
    }

    Ok(Stmt::While { cond, body })
}

fn parse_return_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let expr = parse_expr(pair.into_inner().next().ok_or(ParseError::MissingInner(Rule::return_stmt))?)?;
    Ok(Stmt::Return(expr))
}

fn parse_assign_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let mut inner = pair.into_inner();
    let target = inner.next().ok_or(ParseError::MissingInner(Rule::assign_stmt))?.as_str().to_string();
    let value = parse_expr(inner.next().ok_or(ParseError::MissingInner(Rule::assign_stmt))?)?;

    Ok(Stmt::Assign { target, value })
}

fn parse_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    match pair.as_rule() {
        Rule::expr => parse_expr(pair.into_inner().next().ok_or(ParseError::MissingInner(Rule::expr))?),
        Rule::comparison => parse_comparison(pair),
        Rule::add_expr => parse_add_expr(pair),
        Rule::mul_expr => parse_mul_expr(pair),
        Rule::unary_expr => parse_unary_expr(pair),
        Rule::primary => parse_primary(pair),
        Rule::func_call => parse_func_call(pair),
        Rule::number => parse_number(pair),
        Rule::string => parse_string(pair),
        Rule::boolean => parse_boolean(pair),
        Rule::ident => Ok(Expr::Var(pair.as_str().to_string())),
        rule => Err(ParseError::UnexpectedRule(rule)),
    }
}

fn parse_comparison(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let s = pair.as_str();
    let mut inner = pair.into_inner().peekable();
    let first = inner.next().ok_or(ParseError::MissingInner(Rule::comparison))?;

    // Check if this is just an add_expr (no comparison)
    if inner.peek().is_none() {
        return parse_expr(first);
    }

    let left = parse_expr(first)?;

    // Determine the operator by looking at the original string
    let op = if s.contains(" li suli e ") {
        BinOp::Gt
    } else if s.contains(" li lili e ") {
        BinOp::Lt
    } else if s.contains(" li sama e ") {
        BinOp::Eq
    } else {
        BinOp::Eq
    };

    // Get the second add_expr
    let mut right_pair = None;
    for item in inner {
        if item.as_rule() == Rule::add_expr {
            right_pair = Some(item);
        }
    }

    if let Some(right) = right_pair {
        let right = parse_expr(right)?;
        Ok(Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        })
    } else {
        Ok(left)
    }
}

fn parse_add_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = parse_expr(inner.next().ok_or(ParseError::MissingInner(Rule::add_expr))?)?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "+" => BinOp::Add,
            "-" => BinOp::Sub,
            _ => continue,
        };

        if let Some(right_pair) = inner.next() {
            let right = parse_expr(right_pair)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
    }

    Ok(left)
}

fn parse_mul_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = parse_expr(inner.next().ok_or(ParseError::MissingInner(Rule::mul_expr))?)?;

    while let Some(op_pair) = inner.next() {
        let op = match op_pair.as_str() {
            "*" => BinOp::Mul,
            "/" => BinOp::Div,
            _ => continue,
        };

        if let Some(right_pair) = inner.next() {
            let right = parse_expr(right_pair)?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
    }

    Ok(left)
}

fn parse_unary_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or(ParseError::MissingInner(Rule::unary_expr))?;

    if first.as_str() == "-" {
        let expr = parse_expr(inner.next().ok_or(ParseError::MissingInner(Rule::unary_expr))?)?;
        Ok(Expr::Neg(Box::new(expr)))
    } else {
        parse_expr(first)
    }
}

fn parse_primary(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let inner = pair.into_inner().next().ok_or(ParseError::MissingInner(Rule::primary))?;
    parse_expr(inner)
}

fn parse_func_call(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let name = inner.next().ok_or(ParseError::MissingInner(Rule::func_call))?.as_str().to_string();

    let mut args = Vec::new();
    for item in inner {
        if item.as_rule() == Rule::arg_list {
            for arg in item.into_inner() {
                args.push(parse_expr(arg)?);
            }
        }
    }

    Ok(Expr::FuncCall { name, args })
}

fn parse_number(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let s = pair.as_str();
    s.parse::<f64>()
        .map(Expr::Number)
        .map_err(|_| ParseError::InvalidNumber(s.to_string()))
}

fn parse_string(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let s = pair.as_str();
    let content = s
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(s);
    Ok(Expr::String(content.to_string()))
}

fn parse_boolean(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    match pair.as_str() {
        "lon" => Ok(Expr::Bool(true)),
        "ala" => Ok(Expr::Bool(false)),
        _ => Ok(Expr::Bool(false)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_number() {
        let result = parse("x li jo e 42").unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_parse_string() {
        let result = parse(r#"toki e ("pona")"#).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_parse_func_def() {
        let code = r#"
            ilo sum li pali e (a, b) la open
                pana e a + b
            pini
        "#;
        let result = parse(code).unwrap();
        assert_eq!(result.len(), 1);
    }
}
