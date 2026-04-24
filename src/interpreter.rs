//! Tree-walking interpreter for the Lipona language.
//!
//! Executes AST nodes directly without compilation.
//! Provides scoped variable bindings and runtime value types.

use std::collections::HashMap;
use thiserror::Error;

use crate::ast::{BinOp, Block, Expr, Program, Stmt, StringPart, Type};
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
    /// User-defined function (or lambda).
    ///
    /// `captured` is a snapshot of the scope stack at the time the function
    /// was created. This is used when the function is invoked so that free
    /// variables resolve to the creation-time environment (lexical scoping),
    /// rather than to whatever environment the call site happens to be in.
    Function {
        params: Vec<String>,
        param_types: Vec<Option<Type>>,
        return_type: Option<Type>,
        body: Block,
        captured: Vec<HashMap<String, Value>>,
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

    /// Check whether this value matches the given type annotation.
    ///
    /// Rules:
    /// - `ijo` matches anything (any type)
    /// - otherwise, the value's type name must equal the annotation's name
    pub fn matches_type(&self, ty: &Type) -> bool {
        if *ty == Type::Ijo {
            return true;
        }
        self.type_name() == ty.name()
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
    TypeError { expected: &'static str, got: String },
    #[error("pakala: wrong number of arguments for '{name}' - expected {expected}, got {got}")]
    WrongArity {
        name: String,
        expected: usize,
        got: usize,
    },
    #[error("pakala_toki: function '{func}' parameter '{param}' expected {expected}, got {got}")]
    ParamTypeMismatch {
        func: String,
        param: String,
        expected: String,
        got: String,
    },
    #[error("pakala_toki: function '{func}' expected return type {expected}, got {got}")]
    ReturnTypeMismatch {
        func: String,
        expected: String,
        got: String,
    },
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

    /// Take a snapshot of the current scope stack (used when building a
    /// function value to capture its lexical environment).
    pub fn snapshot(&self) -> Vec<HashMap<String, Value>> {
        self.scopes.clone()
    }

    /// Replace the current scope stack and return the previous one.
    ///
    /// Used to enter a function call with the callee's captured environment,
    /// and to restore the caller's environment on return.
    pub fn replace_scopes(
        &mut self,
        new_scopes: Vec<HashMap<String, Value>>,
    ) -> Vec<HashMap<String, Value>> {
        std::mem::replace(&mut self.scopes, new_scopes)
    }

    /// Return a reference to the current global scope (scope index 0).
    ///
    /// Used when entering a function call to refresh the captured snapshot's
    /// global scope to the live one, so top-level bindings — including the
    /// function itself (for recursion) — remain visible.
    pub fn global_scope(&self) -> &HashMap<String, Value> {
        self.scopes
            .first()
            .expect("Environment must have at least one scope")
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
            Stmt::FuncDef {
                name,
                params,
                param_types,
                return_type,
                body,
            } => {
                // Tentatively bind the function name to ala first, then take
                // an environment snapshot that already includes the new name.
                // This lets the function's own body resolve recursive calls
                // through the captured environment.
                self.env.define(name.clone(), Value::Ala);
                let captured = self.env.snapshot();
                let func = Value::Function {
                    params: params.clone(),
                    param_types: param_types.clone(),
                    return_type: return_type.clone(),
                    body: body.clone(),
                    captured,
                };
                self.env.set(name, func);
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
            (BinOp::Ge, Value::Number(a), Value::Number(b)) => {
                Ok(if a >= b { Value::Bool } else { Value::Ala })
            }
            (BinOp::Le, Value::Number(a), Value::Number(b)) => {
                Ok(if a <= b { Value::Bool } else { Value::Ala })
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
            Value::Function {
                params,
                param_types,
                return_type,
                body,
                captured,
            } => {
                if params.len() != args.len() {
                    return Err(RuntimeError::WrongArity {
                        name: name.to_string(),
                        expected: params.len(),
                        got: args.len(),
                    });
                }

                // Evaluate arguments in current environment
                let evaluated_args = self.eval_args(args)?;

                // Check parameter type annotations (skip when annotation is None)
                for ((param, ty), value) in params
                    .iter()
                    .zip(param_types.iter())
                    .zip(evaluated_args.iter())
                {
                    if let Some(expected) = ty {
                        if !value.matches_type(expected) {
                            return Err(RuntimeError::ParamTypeMismatch {
                                func: name.to_string(),
                                param: param.clone(),
                                expected: expected.to_string(),
                                got: value.type_name().to_string(),
                            });
                        }
                    }
                }

                // Swap in the function's captured environment (lexical
                // scoping). The captured snapshot's global scope (index 0)
                // is refreshed from the caller's current globals so that
                // top-level definitions and mutations made after the
                // function was created — including the function itself for
                // recursion — are still visible inside the call.
                let mut call_scopes = captured;
                if !call_scopes.is_empty() {
                    call_scopes[0] = self.env.global_scope().clone();
                }
                let saved_scopes = self.env.replace_scopes(call_scopes);

                self.env.push_scope();
                for (param, value) in params.iter().zip(evaluated_args) {
                    self.env.define(param.clone(), value);
                }

                // Execute function body
                let result = self.exec_block_in_current_scope(&body);

                // Restore the caller's scope stack.
                self.env.replace_scopes(saved_scopes);

                // Convert result
                let value = result.map(|cf| match cf {
                    ControlFlow::Return(v) => v,
                    ControlFlow::None => Value::Ala,
                })?;

                // Check return type annotation
                if let Some(expected) = &return_type {
                    if !value.matches_type(expected) {
                        return Err(RuntimeError::ReturnTypeMismatch {
                            func: name.to_string(),
                            expected: expected.to_string(),
                            got: value.type_name().to_string(),
                        });
                    }
                }

                Ok(value)
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
