# Lipona - Toki Pona-based Programming Language

## Project Overview

Lipona is a minimal programming language based on Toki Pona grammar structure.
The philosophy is to keep syntax minimal and extend functionality through functions (ilo), not new syntax.

## Language Specification

See `Lipona.md` for the full MVP specification.

## Project Structure

```
lipona/
├── src/
│   ├── main.rs        # Entry point
│   ├── parser.rs      # AST parser
│   ├── ast.rs         # AST definitions
│   ├── interpreter.rs # Tree-walking interpreter
│   ├── stdlib.rs      # Standard library (ilo)
│   └── lipona.pest    # PEG grammar
├── examples/          # .lipo example files
├── tests/             # Integration tests
├── Lipona.md          # Language specification
└── CLAUDE.md          # This file
```

## Development Commands

```bash
# Enter dev environment
nix develop

# Build
cargo build

# Run interpreter
cargo run -- <file.lipo>

# Run with REPL (future)
cargo run

# Test
cargo test

# Watch mode
cargo watch -x run
```

## Key Concepts

### Reserved Keywords
`la`, `open`, `pini`, `ilo`, `pana`, `wile`, `taso`, `suli`, `lili`, `suli_sama`, `lili_sama`, `sama`, `jo`, `lon`, `ala`

### Core Syntax Patterns
- Assignment: `x jo Expr`
- Function call: `NAME(args)`
- Function def: `ilo NAME (params) open ... pini`
- If/else: `Cond la open ... pini taso open ... pini`
- While: `wile Cond la open ... pini`
- Return: `pana Expr`
- Compare: `x suli y` (>), `x lili y` (<), `x sama y` (==), `x suli_sama y` (>=), `x lili_sama y` (<=)

### Types
- Number: `10`, `3.14`
- String: `"pona"`
- Boolean: `lon` (true), `ala` (false/null)

## Implementation Notes

- File extension: `.lipo`
- Use `pest` for parsing
- Tree-walking interpreter for MVP
- All stdlib functions are regular `ilo` (no special syntax)

## Testing Strategy

1. Unit tests for lexer tokens
2. Unit tests for parser AST output
3. Integration tests running `.lipo` files
4. Test each stdlib function
