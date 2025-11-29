use std::collections::HashMap;

use crate::interpreter::{RuntimeError, Value};

/// Standard library function signature
type StdLibFn = fn(Vec<Value>) -> Result<Value, RuntimeError>;

/// Standard library functions
pub struct StdLib {
    functions: HashMap<&'static str, StdLibFn>,
}

impl StdLib {
    pub fn new() -> Self {
        let mut functions: HashMap<&'static str, StdLibFn> = HashMap::new();

        // I/O
        functions.insert("toki", stdlib_toki);

        // Number
        functions.insert("nanpa_sin", stdlib_nanpa_sin);
        functions.insert("nanpa_len", stdlib_nanpa_len);

        // String
        functions.insert("sitelen_len", stdlib_sitelen_len);
        functions.insert("sitelen_sama", stdlib_sitelen_sama);

        // List
        functions.insert("kulupu_sin", stdlib_kulupu_sin);
        functions.insert("kulupu_len", stdlib_kulupu_len);
        functions.insert("kulupu_ken", stdlib_kulupu_ken);
        functions.insert("kulupu_lon", stdlib_kulupu_lon);
        functions.insert("kulupu_aksen", stdlib_kulupu_aksen);

        // Map
        functions.insert("nasin_sin", stdlib_nasin_sin);
        functions.insert("nasin_ken", stdlib_nasin_ken);
        functions.insert("nasin_lon", stdlib_nasin_lon);

        Self { functions }
    }

    pub fn has_function(&self, name: &str) -> bool {
        self.functions.contains_key(name)
    }

    pub fn call(&self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        if let Some(func) = self.functions.get(name) {
            func(args)
        } else {
            Err(RuntimeError::UndefinedFunction(name.to_string()))
        }
    }
}

impl Default for StdLib {
    fn default() -> Self {
        Self::new()
    }
}

// === I/O ===

/// toki e (x) - print
fn stdlib_toki(args: Vec<Value>) -> Result<Value, RuntimeError> {
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            print!(" ");
        }
        print!("{arg}");
    }
    println!();
    Ok(Value::Ala)
}

// === Number ===

/// nanpa_sin e (x) - string to number
fn stdlib_nanpa_sin(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("nanpa_sin", &args, 1)?;
    match &args[0] {
        Value::String(s) => s
            .parse::<f64>()
            .map(Value::Number)
            .map_err(|_| RuntimeError::TypeError {
                expected: "valid number string",
                got: "invalid string".to_string(),
            }),
        Value::Number(n) => Ok(Value::Number(*n)),
        other => Err(RuntimeError::TypeError {
            expected: "sitelen",
            got: other.type_name().to_string(),
        }),
    }
}

/// nanpa_len e (x) - number of digits
fn stdlib_nanpa_len(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("nanpa_len", &args, 1)?;
    match &args[0] {
        Value::Number(n) => {
            if n.is_nan() || n.is_infinite() {
                return Err(RuntimeError::TypeError {
                    expected: "finite number",
                    got: format!("{n}"),
                });
            }
            let abs = n.abs();
            let len = if abs < 1.0 {
                1 // 0.xxx is considered 1 digit for integer part
            } else {
                (abs.log10().floor() as usize) + 1
            };
            Ok(Value::Number(len as f64))
        }
        other => Err(RuntimeError::TypeError {
            expected: "nanpa",
            got: other.type_name().to_string(),
        }),
    }
}

// === String ===

/// sitelen_len e (s) - string length
fn stdlib_sitelen_len(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("sitelen_len", &args, 1)?;
    match &args[0] {
        Value::String(s) => Ok(Value::Number(s.chars().count() as f64)),
        other => Err(RuntimeError::TypeError {
            expected: "sitelen",
            got: other.type_name().to_string(),
        }),
    }
}

/// sitelen_sama e (a, b) - string equality
fn stdlib_sitelen_sama(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("sitelen_sama", &args, 2)?;
    match (&args[0], &args[1]) {
        (Value::String(a), Value::String(b)) => Ok(Value::Bool(a == b)),
        (Value::String(_), other) | (other, _) => Err(RuntimeError::TypeError {
            expected: "sitelen",
            got: other.type_name().to_string(),
        }),
    }
}

// === List ===

/// kulupu_sin e (...items) - create list
fn stdlib_kulupu_sin(args: Vec<Value>) -> Result<Value, RuntimeError> {
    Ok(Value::List(args))
}

/// kulupu_len e (arr) - list length
fn stdlib_kulupu_len(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("kulupu_len", &args, 1)?;
    match &args[0] {
        Value::List(items) => Ok(Value::Number(items.len() as f64)),
        other => Err(RuntimeError::TypeError {
            expected: "kulupu",
            got: other.type_name().to_string(),
        }),
    }
}

/// kulupu_ken e (arr, i) - get element
fn stdlib_kulupu_ken(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("kulupu_ken", &args, 2)?;
    match (&args[0], &args[1]) {
        (Value::List(items), Value::Number(i)) => {
            let index = to_index(*i)?;
            if index >= items.len() {
                Ok(Value::Ala)
            } else {
                Ok(items[index].clone())
            }
        }
        (Value::List(_), other) => Err(RuntimeError::TypeError {
            expected: "nanpa",
            got: other.type_name().to_string(),
        }),
        (other, _) => Err(RuntimeError::TypeError {
            expected: "kulupu",
            got: other.type_name().to_string(),
        }),
    }
}

/// kulupu_lon e (arr, i, val) - set element
fn stdlib_kulupu_lon(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("kulupu_lon", &args, 3)?;
    match (&args[0], &args[1]) {
        (Value::List(items), Value::Number(i)) => {
            let index = to_index(*i)?;
            if index >= items.len() {
                Err(RuntimeError::IndexOutOfBounds {
                    index,
                    len: items.len(),
                })
            } else {
                let mut new_items = items.clone();
                new_items[index] = args[2].clone();
                Ok(Value::List(new_items))
            }
        }
        (Value::List(_), other) => Err(RuntimeError::TypeError {
            expected: "nanpa",
            got: other.type_name().to_string(),
        }),
        (other, _) => Err(RuntimeError::TypeError {
            expected: "kulupu",
            got: other.type_name().to_string(),
        }),
    }
}

/// kulupu_aksen e (arr, val) - append
fn stdlib_kulupu_aksen(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("kulupu_aksen", &args, 2)?;
    match &args[0] {
        Value::List(items) => {
            let mut new_items = items.clone();
            new_items.push(args[1].clone());
            Ok(Value::List(new_items))
        }
        other => Err(RuntimeError::TypeError {
            expected: "kulupu",
            got: other.type_name().to_string(),
        }),
    }
}

// === Map ===

/// nasin_sin e () - create empty map
fn stdlib_nasin_sin(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("nasin_sin", &args, 0)?;
    Ok(Value::Map(HashMap::new()))
}

/// nasin_ken e (m, key) - get value
fn stdlib_nasin_ken(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("nasin_ken", &args, 2)?;
    match (&args[0], &args[1]) {
        (Value::Map(map), Value::String(key)) => {
            Ok(map.get(key).cloned().unwrap_or(Value::Ala))
        }
        (Value::Map(_), other) => Err(RuntimeError::TypeError {
            expected: "sitelen",
            got: other.type_name().to_string(),
        }),
        (other, _) => Err(RuntimeError::TypeError {
            expected: "nasin",
            got: other.type_name().to_string(),
        }),
    }
}

/// nasin_lon e (m, key, val) - set value
fn stdlib_nasin_lon(args: Vec<Value>) -> Result<Value, RuntimeError> {
    check_arity("nasin_lon", &args, 3)?;
    match (&args[0], &args[1]) {
        (Value::Map(map), Value::String(key)) => {
            let mut new_map = map.clone();
            new_map.insert(key.clone(), args[2].clone());
            Ok(Value::Map(new_map))
        }
        (Value::Map(_), other) => Err(RuntimeError::TypeError {
            expected: "sitelen",
            got: other.type_name().to_string(),
        }),
        (other, _) => Err(RuntimeError::TypeError {
            expected: "nasin",
            got: other.type_name().to_string(),
        }),
    }
}

// === Helper ===

fn check_arity(name: &str, args: &[Value], expected: usize) -> Result<(), RuntimeError> {
    if args.len() != expected {
        Err(RuntimeError::WrongArity {
            name: name.to_string(),
            expected,
            got: args.len(),
        })
    } else {
        Ok(())
    }
}

/// Convert f64 to usize for indexing, validating it's a non-negative integer
fn to_index(n: f64) -> Result<usize, RuntimeError> {
    if n < 0.0 || n.is_nan() || n.is_infinite() || n.fract() != 0.0 {
        return Err(RuntimeError::TypeError {
            expected: "non-negative integer",
            got: format!("{n}"),
        });
    }
    if n > (usize::MAX as f64) {
        return Err(RuntimeError::TypeError {
            expected: "index within bounds",
            got: format!("{n} exceeds maximum index"),
        });
    }
    Ok(n as usize)
}
