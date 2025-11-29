//! Abstract Syntax Tree definitions for the Lipona language.
//!
//! This module defines the core AST types used by the parser and interpreter:
//! - [`Expr`]: Expression nodes (literals, variables, operations, function calls)
//! - [`Stmt`]: Statement nodes (assignments, control flow, function definitions)
//! - [`BinOp`]: Binary operators

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add,      // +
    Sub,      // -
    Mul,      // *
    Div,      // /
    Gt,       // suli (>)
    Lt,       // lili (<)
    Ge,       // suli_sama (>=)
    Le,       // lili_sama (<=)
    Eq,       // sama (==)
}

/// A part of a template string
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    /// Literal text
    Literal(String),
    /// Interpolated expression: {expr}
    Interpolation(Box<Expr>),
}

/// Expression AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Number literal: 10, 3.14
    Number(f64),
    /// Template string: "Hello, {name}!"
    TemplateString(Vec<StringPart>),
    /// Boolean: lon (true), ala (false/null)
    Bool(bool),
    /// Variable reference
    Var(String),
    /// Binary operation
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    /// Unary negation
    Neg(Box<Expr>),
    /// Function call: NAME e (args)
    FuncCall {
        name: String,
        args: Vec<Expr>,
    },
}

/// Statement AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Assignment: x li jo e Expr
    Assign {
        target: String,
        value: Expr,
    },
    /// If statement: Cond la open ... pini taso open ... pini
    If {
        cond: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    /// While loop: wile Cond la open ... pini
    While {
        cond: Expr,
        body: Block,
    },
    /// Function definition: ilo NAME li pali e (params) la open ... pini
    FuncDef {
        name: String,
        params: Vec<String>,
        body: Block,
    },
    /// Return statement: pana e Expr
    Return(Expr),
    /// Expression statement (for side effects like function calls)
    Expr(Expr),
}

/// A block is a sequence of statements
pub type Block = Vec<Stmt>;

/// A program is a sequence of statements
pub type Program = Vec<Stmt>;
