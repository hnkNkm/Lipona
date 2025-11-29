//! Tree-walking interpreter for the Lipona language.
//!
//! Executes AST nodes directly without compilation.
//! Provides scoped variable bindings and runtime value types.

use std::collections::HashMap;
use thiserror::Error;

use crate::ast::{BinOp, Block, Expr, Program, Stmt, StringPart};
use crate::stdlib::StdLib;

/// Runtime value
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(f64),
    String(String),
    /// lon (true) - only true is represented as Bool
    Bool,
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
            Value::Bool => true,
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
            Value::Bool => "lon",
            Value::List(_) => "kulupu",
            Value::Map(_) => "nasin",
            Value::Ala => "ala",
            Value::Function { .. } => "ilo",
        }
    }
}

/// Maximum safe integer that can be exactly represented in f64 (2^53)
pub const F64_SAFE_INT_MAX: f64 = 9_007_199_254_740_992.0;
pub const F64_SAFE_INT_MIN: f64 = -9_007_199_254_740_992.0;

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(n) => {
                // Only display as integer if it's a whole number within safe range
                if n.fract() == 0.0 && *n >= F64_SAFE_INT_MIN && *n <= F64_SAFE_INT_MAX {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{n}")
                }
            }
            Value::String(s) => write!(f, "{s}"),
            Value::Bool => write!(f, "lon"),
            Value::List(items) => {
                let strs: Vec<String> = items.iter().map(|v| format!("{v}")).collect();
                write!(f, "[{}]", strs.join(", "))
            }
            Value::Map(m) => {
                let mut strs: Vec<String> = m.iter().map(|(k, v)| format!("{k}: {v}")).collect();
                strs.sort();
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
    #[error("pakala: loop iteration limit exceeded (possible infinite loop)")]
    InfiniteLoop,
    #[error("pakala: maximum call depth exceeded (possible infinite recursion)")]
    StackOverflow,
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
        debug_assert!(self.scopes.len() > 1, "attempted to pop global scope");
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.scopes
            .last_mut()
            .expect("Environment must have at least one scope")
            .insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Some(value);
            }
        }
        None
    }

    pub fn set(&mut self, name: &str, value: Value) {
        // Search all scopes from innermost to outermost
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return;
            }
        }
        // If not found, define in current scope
        self.define(name.to_string(), value);
    }

    /// Create an isolated environment for function calls.
    /// Returns saved scopes that must be restored after function execution.
    pub fn isolate_for_function(&mut self) -> Vec<HashMap<String, Value>> {
        let saved_scopes = std::mem::take(&mut self.scopes);
        // Keep only global scope for function execution
        self.scopes = vec![saved_scopes[0].clone()];
        saved_scopes
    }

    /// Restore scopes after function execution.
    pub fn restore_scopes(&mut self, saved_scopes: Vec<HashMap<String, Value>>) {
        self.scopes = saved_scopes;
    }
}

impl Default for Environment {
    fn default() -> Self {
        Self::new()
    }
}

/// Maximum iterations for a single while loop
const MAX_LOOP_ITERATIONS: u64 = 10_000_000;

/// Maximum call stack depth
const MAX_CALL_DEPTH: usize = 1000;

/// The interpreter
pub struct Interpreter {
    env: Environment,
    stdlib: StdLib,
    call_depth: usize,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            env: Environment::new(),
            stdlib: StdLib::new(),
            call_depth: 0,
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
                let mut iterations: u64 = 0;
                while self.eval_expr(cond)?.is_truthy() {
                    iterations += 1;
                    if iterations > MAX_LOOP_ITERATIONS {
                        return Err(RuntimeError::InfiniteLoop);
                    }
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
        let result = self.exec_block_in_current_scope(block);
        self.env.pop_scope();
        result
    }

    /// Execute a block in the current scope without creating a new one.
    /// Used when the caller has already set up the scope (e.g., in function calls).
    fn exec_block_in_current_scope(&mut self, block: &Block) -> Result<ControlFlow, RuntimeError> {
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
            Expr::TemplateString(parts) => self.eval_template_string(parts),
            // In Lipona, `lon` (true) is Value::Bool, `ala` (false) is Value::Ala
            Expr::Bool(b) => Ok(if *b { Value::Bool } else { Value::Ala }),
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

    fn eval_template_string(&mut self, parts: &[StringPart]) -> Result<Value, RuntimeError> {
        let mut result = String::new();
        for part in parts {
            match part {
                StringPart::Literal(s) => result.push_str(s),
                StringPart::Interpolation(expr) => {
                    let value = self.eval_expr(expr)?;
                    result.push_str(&format!("{value}"));
                }
            }
        }
        Ok(Value::String(result))
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
                Ok(Value::String(format!("{a}{b}")))
            }

            // Comparisons - return Bool for true, Ala for false
            (BinOp::Gt, Value::Number(a), Value::Number(b)) => {
                Ok(if a > b { Value::Bool } else { Value::Ala })
            }
            (BinOp::Lt, Value::Number(a), Value::Number(b)) => {
                Ok(if a < b { Value::Bool } else { Value::Ala })
            }
            (BinOp::Eq, a, b) => Ok(if a == b { Value::Bool } else { Value::Ala }),

            // Type errors
            _ => Err(RuntimeError::TypeError {
                expected: "compatible types",
                got: format!("{} and {}", left_val.type_name(), right_val.type_name()),
            }),
        }
    }

    fn call_function(&mut self, name: &str, args: &[Expr]) -> Result<Value, RuntimeError> {
        // Check call depth limit
        self.call_depth += 1;
        if self.call_depth > MAX_CALL_DEPTH {
            self.call_depth -= 1;
            return Err(RuntimeError::StackOverflow);
        }

        let result = self.call_function_inner(name, args);
        self.call_depth -= 1;
        result
    }

    fn call_function_inner(&mut self, name: &str, args: &[Expr]) -> Result<Value, RuntimeError> {
        // Check stdlib first
        if self.stdlib.has_function(name) {
            let evaluated_args = self.eval_args(args)?;
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

                // Evaluate arguments in current environment
                let evaluated_args = self.eval_args(args)?;

                // Isolate environment for function execution (only global scope visible)
                let saved_scopes = self.env.isolate_for_function();

                // Create function scope and bind parameters
                self.env.push_scope();
                for (param, value) in params.iter().zip(evaluated_args) {
                    self.env.define(param.clone(), value);
                }

                // Execute function body
                let result = self.exec_block_in_current_scope(&body);

                // Restore original scopes
                self.env.restore_scopes(saved_scopes);

                // Convert result
                result.map(|cf| match cf {
                    ControlFlow::Return(v) => v,
                    ControlFlow::None => Value::Ala,
                })
            }
            _ => Err(RuntimeError::TypeError {
                expected: "ilo",
                got: func.type_name().to_string(),
            }),
        }
    }

    fn eval_args(&mut self, args: &[Expr]) -> Result<Vec<Value>, RuntimeError> {
        args.iter().map(|arg| self.eval_expr(arg)).collect()
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
