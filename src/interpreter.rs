use std::collections::HashMap;
use thiserror::Error;

use crate::ast::{BinOp, Block, Expr, Program, Stmt};
use crate::stdlib::StdLib;

/// Runtime value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
    /// ala represents null/false/empty
    Ala,
    /// User-defined function
    Function {
        params: Vec<String>,
        body: Block,
    },
}

impl Value {
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            Value::Ala => false,
            Value::Number(n) => !n.is_nan() && *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::List(l) => !l.is_empty(),
            Value::Map(m) => !m.is_empty(),
            Value::Function { .. } => true,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "nanpa",
            Value::String(_) => "sitelen",
            Value::Bool(_) => "lon/ala",
            Value::List(_) => "kulupu",
            Value::Map(_) => "nasin",
            Value::Ala => "ala",
            Value::Function { .. } => "ilo",
        }
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                if n.fract() == 0.0 && n.abs() < (i64::MAX as f64) {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Value::String(s) => write!(f, "{}", s),
            Value::Bool(b) => write!(f, "{}", if *b { "lon" } else { "ala" }),
            Value::List(items) => {
                let strs: Vec<String> = items.iter().map(|v| format!("{}", v)).collect();
                write!(f, "[{}]", strs.join(", "))
            }
            Value::Map(m) => {
                let strs: Vec<String> = m.iter().map(|(k, v)| format!("{}: {}", k, v)).collect();
                write!(f, "{{{}}}", strs.join(", "))
            }
            Value::Ala => write!(f, "ala"),
            Value::Function { params, .. } => write!(f, "<ilo({})>", params.join(", ")),
        }
    }
}

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("pakala: undefined variable '{0}'")]
    UndefinedVariable(String),
    #[error("pakala: undefined function '{0}'")]
    UndefinedFunction(String),
    #[error("pakala: division by zero")]
    DivisionByZero,
    #[error("pakala: type error - expected {expected}, got {got}")]
    TypeError {
        expected: &'static str,
        got: String,
    },
    #[error("pakala: wrong number of arguments for '{name}' - expected {expected}, got {got}")]
    WrongArity { name: String, expected: usize, got: usize },
    #[error("pakala: index out of bounds - {index} >= {len}")]
    IndexOutOfBounds { index: usize, len: usize },
    #[allow(dead_code)]
    #[error("pakala: key not found '{0}'")]
    KeyNotFound(String),
}

/// Control flow signals
enum ControlFlow {
    None,
    Return(Value),
}

/// Environment for variable bindings
#[derive(Debug, Clone)]
pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    pub fn set(&mut self, name: &str, value: Value) -> bool {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return true;
            }
        }
        // If not found, define in current scope
        self.define(name.to_string(), value);
        true
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// The interpreter
pub struct Interpreter {
    env: Environment,
    stdlib: StdLib,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            stdlib: StdLib::new(),
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<Value, RuntimeError> {
        for stmt in program {
            match self.exec_stmt(stmt)? {
                ControlFlow::Return(v) => return Ok(v),
                ControlFlow::None => {}
            }
        }
        Ok(Value::Ala)
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, RuntimeError> {
        match stmt {
            Stmt::Assign { target, value } => {
                let val = self.eval_expr(value)?;
                self.env.set(target, val);
                Ok(ControlFlow::None)
            }
            Stmt::If {
                cond,
                then_block,
                else_block,
            } => {
                let cond_val = self.eval_expr(cond)?;
                if cond_val.is_truthy() {
                    self.exec_block(then_block)
                } else if let Some(else_b) = else_block {
                    self.exec_block(else_b)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            Stmt::While { cond, body } => {
                while self.eval_expr(cond)?.is_truthy() {
                    if let ControlFlow::Return(v) = self.exec_block(body)? {
                        return Ok(ControlFlow::Return(v));
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::FuncDef { name, params, body } => {
                let func = Value::Function {
                    params: params.clone(),
                    body: body.clone(),
                };
                self.env.define(name.clone(), func);
                Ok(ControlFlow::None)
            }
            Stmt::Return(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(ControlFlow::Return(val))
            }
            Stmt::Expr(expr) => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }
        }
    }

    fn exec_block(&mut self, block: &Block) -> Result<ControlFlow, RuntimeError> {
        self.env.push_scope();
        for stmt in block {
            if let ControlFlow::Return(v) = self.exec_stmt(stmt)? {
                self.env.pop_scope();
                return Ok(ControlFlow::Return(v));
            }
        }
        self.env.pop_scope();
        Ok(ControlFlow::None)
    }

    /// Execute a block without creating a new scope
    /// Used when the caller has already set up the scope (e.g., in function calls)
    fn exec_block_no_scope(&mut self, block: &Block) -> Result<ControlFlow, RuntimeError> {
        for stmt in block {
            if let ControlFlow::Return(v) = self.exec_stmt(stmt)? {
                return Ok(ControlFlow::Return(v));
            }
        }
        Ok(ControlFlow::None)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Number(n) => Ok(Value::Number(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Bool(b) => {
                if *b {
                    Ok(Value::Bool(true))
                } else {
                    Ok(Value::Ala)
                }
            }
            Expr::Var(name) => self
                .env
                .get(name)
                .cloned()
                .ok_or_else(|| RuntimeError::UndefinedVariable(name.clone())),
            Expr::Neg(inner) => {
                let val = self.eval_expr(inner)?;
                match val {
                    Value::Number(n) => Ok(Value::Number(-n)),
                    _ => Err(RuntimeError::TypeError {
                        expected: "nanpa",
                        got: val.type_name().to_string(),
                    }),
                }
            }
            Expr::Binary { left, op, right } => self.eval_binary(left, op, right),
            Expr::FuncCall { name, args } => self.call_function(name, args),
        }
    }

    fn eval_binary(
        &mut self,
        left: &Expr,
        op: &BinOp,
        right: &Expr,
    ) -> Result<Value, RuntimeError> {
        let left_val = self.eval_expr(left)?;
        let right_val = self.eval_expr(right)?;

        match (op, &left_val, &right_val) {
            // Numeric operations
            (BinOp::Add, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (BinOp::Sub, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
            (BinOp::Mul, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
            (BinOp::Div, Value::Number(_), Value::Number(b)) if *b == 0.0 => {
                Err(RuntimeError::DivisionByZero)
            }
            (BinOp::Div, Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),

            // String concatenation
            (BinOp::Add, Value::String(a), Value::String(b)) => {
                Ok(Value::String(format!("{}{}", a, b)))
            }

            // Comparisons
            (BinOp::Gt, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
            (BinOp::Lt, Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
            (BinOp::Eq, a, b) => Ok(Value::Bool(a == b)),

            // Type errors
            _ => Err(RuntimeError::TypeError {
                expected: "compatible types",
                got: format!("{} and {}", left_val.type_name(), right_val.type_name()),
            }),
        }
    }

    fn call_function(&mut self, name: &str, args: &[Expr]) -> Result<Value, RuntimeError> {
        // Check stdlib first
        if self.stdlib.has_function(name) {
            let mut evaluated_args = Vec::new();
            for arg in args {
                evaluated_args.push(self.eval_expr(arg)?);
            }
            return self.stdlib.call(name, evaluated_args);
        }

        // Check user-defined functions
        let func = self
            .env
            .get(name)
            .cloned()
            .ok_or_else(|| RuntimeError::UndefinedFunction(name.to_string()))?;

        match func {
            Value::Function { params, body } => {
                if params.len() != args.len() {
                    return Err(RuntimeError::WrongArity {
                        name: name.to_string(),
                        expected: params.len(),
                        got: args.len(),
                    });
                }

                // Evaluate arguments
                let mut evaluated_args = Vec::new();
                for arg in args {
                    evaluated_args.push(self.eval_expr(arg)?);
                }

                // Create new scope and bind parameters
                self.env.push_scope();
                for (param, value) in params.iter().zip(evaluated_args) {
                    self.env.define(param.clone(), value);
                }

                // Execute function body
                let result = match self.exec_block_no_scope(&body)? {
                    ControlFlow::Return(v) => v,
                    ControlFlow::None => Value::Ala,
                };

                self.env.pop_scope();
                Ok(result)
            }
            _ => Err(RuntimeError::TypeError {
                expected: "ilo",
                got: func.type_name().to_string(),
            }),
        }
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
