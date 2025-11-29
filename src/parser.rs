//! Parser for the Lipona language.
//!
//! Uses pest PEG parser to convert source code into an AST.
//! The grammar is defined in `lipona.pest`.

use pest::Parser;
use pest_derive::Parser;
use thiserror::Error;

use crate::ast::{BinOp, Block, Expr, Program, Stmt, StringPart};

#[derive(Parser)]
#[grammar = "lipona.pest"]
pub struct LiponaParser;

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Parse error: {0}")]
    Pest(Box<pest::error::Error<Rule>>),
    #[error("Unexpected rule: {0:?}")]
    UnexpectedRule(Rule),
    #[error("Invalid number: {0}")]
    InvalidNumber(String),
    #[error("Invalid boolean: {0}")]
    InvalidBoolean(String),
    #[error("Parse error: missing inner element in {0:?}")]
    MissingInner(Rule),
}

impl From<pest::error::Error<Rule>> for ParseError {
    fn from(err: pest::error::Error<Rule>) -> Self {
        ParseError::Pest(Box::new(err))
    }
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
            rule => {
                return Err(ParseError::UnexpectedRule(rule));
            }
        }
    }

    Ok(Stmt::FuncDef { name, params, body })
}

fn parse_if_stmt(pair: pest::iterators::Pair<Rule>) -> Result<Stmt, ParseError> {
    let mut inner = pair.into_inner();
    let cond = parse_expr(inner.next().ok_or(ParseError::MissingInner(Rule::if_stmt))?)?;

    let mut then_block: Block = Vec::new();
    let mut else_block: Option<Block> = None;

    for item in inner {
        match item.as_rule() {
            Rule::stmt => {
                then_block.push(parse_stmt(item)?);
            }
            Rule::else_block => {
                let mut else_stmts = Vec::new();
                for else_item in item.into_inner() {
                    if else_item.as_rule() == Rule::stmt {
                        else_stmts.push(parse_stmt(else_item)?);
                    }
                }
                else_block = Some(else_stmts);
            }
            rule => {
                return Err(ParseError::UnexpectedRule(rule));
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
        match item.as_rule() {
            Rule::stmt => body.push(parse_stmt(item)?),
            Rule::EOI => {}
            rule => return Err(ParseError::UnexpectedRule(rule)),
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
    let mut inner = pair.into_inner();
    let first = inner.next().ok_or(ParseError::MissingInner(Rule::comparison))?;

    // Check if there's a comp_op (comparison operator)
    let Some(comp_op) = inner.next() else {
        // No comparison operator, just return the add_expr
        return parse_expr(first);
    };

    // Validate comp_op rule
    if comp_op.as_rule() != Rule::comp_op {
        return Err(ParseError::UnexpectedRule(comp_op.as_rule()));
    }

    let left = parse_expr(first)?;

    // Extract the comparison kind from comp_op
    let op = {
        let comp_kind = comp_op
            .into_inner()
            .find(|item| item.as_rule() == Rule::comp_kind)
            .ok_or(ParseError::MissingInner(Rule::comp_op))?;
        match comp_kind.as_str() {
            "suli" => BinOp::Gt,
            "lili" => BinOp::Lt,
            "suli_sama" => BinOp::Ge,
            "lili_sama" => BinOp::Le,
            "sama" => BinOp::Eq,
            _ => return Err(ParseError::UnexpectedRule(Rule::comp_kind)),
        }
    };

    // Get the right operand
    let right_pair = inner.next().ok_or(ParseError::MissingInner(Rule::comparison))?;
    let right = parse_expr(right_pair)?;

    Ok(Expr::Binary {
        left: Box::new(left),
        op,
        right: Box::new(right),
    })
}

fn parse_binary_expr(
    pair: pest::iterators::Pair<Rule>,
    rule: Rule,
    op_mapper: fn(&str) -> Option<BinOp>,
) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner();
    let mut left = parse_expr(inner.next().ok_or(ParseError::MissingInner(rule))?)?;

    while let Some(op_pair) = inner.next() {
        let Some(op) = op_mapper(op_pair.as_str()) else {
            return Err(ParseError::UnexpectedRule(op_pair.as_rule()));
        };

        let right_pair = inner.next().ok_or(ParseError::MissingInner(rule))?;
        let right = parse_expr(right_pair)?;
        left = Expr::Binary {
            left: Box::new(left),
            op,
            right: Box::new(right),
        };
    }

    Ok(left)
}

fn parse_add_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    parse_binary_expr(pair, Rule::add_expr, |s| match s {
        "+" => Some(BinOp::Add),
        "-" => Some(BinOp::Sub),
        _ => None,
    })
}

fn parse_mul_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    parse_binary_expr(pair, Rule::mul_expr, |s| match s {
        "*" => Some(BinOp::Mul),
        "/" => Some(BinOp::Div),
        _ => None,
    })
}

fn parse_unary_expr(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut inner = pair.into_inner().peekable();

    // Check if there's a negation operator by peeking at the first element
    let is_negated = inner.peek().is_some_and(|p| p.as_str() == "-");

    if is_negated {
        inner.next(); // consume the "-"
        let primary = inner.next().ok_or(ParseError::MissingInner(Rule::unary_expr))?;
        let expr = parse_expr(primary)?;
        Ok(Expr::Neg(Box::new(expr)))
    } else {
        let primary = inner.next().ok_or(ParseError::MissingInner(Rule::unary_expr))?;
        parse_expr(primary)
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
        match item.as_rule() {
            Rule::arg_list => {
                for arg in item.into_inner() {
                    args.push(parse_expr(arg)?);
                }
            }
            rule => return Err(ParseError::UnexpectedRule(rule)),
        }
    }

    Ok(Expr::FuncCall { name, args })
}

fn parse_number(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let s = pair.as_str();
    let n = s.parse::<f64>()
        .map_err(|_| ParseError::InvalidNumber(s.to_string()))?;

    if !n.is_finite() {
        return Err(ParseError::InvalidNumber(s.to_string()));
    }

    Ok(Expr::Number(n))
}

fn parse_string(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    let mut parts = Vec::new();

    for inner in pair.into_inner() {
        match inner.as_rule() {
            Rule::string_inner => {
                let part = inner.into_inner().next().ok_or(ParseError::MissingInner(Rule::string_inner))?;
                match part.as_rule() {
                    Rule::interpolation => {
                        let expr_pair = part.into_inner().next().ok_or(ParseError::MissingInner(Rule::interpolation))?;
                        let expr = parse_expr(expr_pair)?;
                        parts.push(StringPart::Interpolation(Box::new(expr)));
                    }
                    Rule::string_literal => {
                        let unescaped = unescape_string(part.as_str());
                        parts.push(StringPart::Literal(unescaped));
                    }
                    rule => return Err(ParseError::UnexpectedRule(rule)),
                }
            }
            rule => return Err(ParseError::UnexpectedRule(rule)),
        }
    }

    Ok(Expr::TemplateString(parts))
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('r') => result.push('\r'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

fn parse_boolean(pair: pest::iterators::Pair<Rule>) -> Result<Expr, ParseError> {
    match pair.as_str() {
        "lon" => Ok(Expr::Bool(true)),
        "ala" => Ok(Expr::Bool(false)),
        other => Err(ParseError::InvalidBoolean(other.to_string())),
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
