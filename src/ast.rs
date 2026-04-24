//! Abstract Syntax Tree definitions for the Lipona language.
//!
//! This module defines the core AST types used by the parser and interpreter:
//! - [`Expr`]: Expression nodes (literals, variables, operations, function calls)
//! - [`Stmt`]: Statement nodes (assignments, control flow, function definitions)
//! - [`BinOp`]: Binary operators

/// Type annotation (Toki Pona type categories).
///
/// Used in function signatures for optional type checking. Corresponds to
/// `Value::type_name()` strings. `Ijo` acts as `any` — skips type checking.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Type {
    /// nanpa - Number
    Nanpa,
    /// sitelen - String
    Sitelen,
    /// lon - Boolean (true). Also accepts ala as a falsy value.
    Lon,
    /// kulupu - List
    Kulupu,
    /// nasin - Map
    Nasin,
    /// ilo - Function
    Ilo,
    /// ala - Null / absent value
    Ala,
    /// ijo - Any (skips type check)
    Ijo,
}

impl Type {
    /// Parse a type name from a string. Returns None if not a valid type name.
    pub fn from_name(s: &str) -> Option<Self> {
        match s {
            "nanpa" => Some(Type::Nanpa),
            "sitelen" => Some(Type::Sitelen),
            "lon" => Some(Type::Lon),
            "kulupu" => Some(Type::Kulupu),
            "nasin" => Some(Type::Nasin),
            "ilo" => Some(Type::Ilo),
            "ala" => Some(Type::Ala),
            "ijo" => Some(Type::Ijo),
            _ => None,
        }
    }

    /// Human-readable name (matches Value::type_name()).
    pub fn name(&self) -> &'static str {
        match self {
            Type::Nanpa => "nanpa",
            Type::Sitelen => "sitelen",
            Type::Lon => "lon",
            Type::Kulupu => "kulupu",
            Type::Nasin => "nasin",
            Type::Ilo => "ilo",
            Type::Ala => "ala",
            Type::Ijo => "ijo",
        }
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinOp {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Gt,  // suli (>)
    Lt,  // lili (<)
    Ge,  // suli_sama (>=)
    Le,  // lili_sama (<=)
    Eq,  // sama (==)
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
    FuncCall { name: String, args: Vec<Expr> },
    /// Anonymous function literal (lambda): ilo (params) [-> type] open ... pini
    ///
    /// Evaluates to a `Value::Function` whose `captured` field is a snapshot
    /// of the enclosing scope stack.
    Lambda {
        params: Vec<String>,
        param_types: Vec<Option<Type>>,
        return_type: Option<Type>,
        body: Block,
    },
}

/// Statement AST node
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// Assignment: x li jo e Expr
    Assign { target: String, value: Expr },
    /// If statement: Cond la open ... pini taso open ... pini
    If {
        cond: Expr,
        then_block: Block,
        else_block: Option<Block>,
    },
    /// While loop: wile Cond la open ... pini
    While { cond: Expr, body: Block },
    /// Function definition: ilo NAME (params) open ... pini
    ///
    /// Each parameter may have an optional type annotation (written as
    /// `name: type` in source). `return_type` corresponds to the optional
    /// `-> type` suffix on the function signature.
    FuncDef {
        name: String,
        params: Vec<String>,
        param_types: Vec<Option<Type>>,
        return_type: Option<Type>,
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
