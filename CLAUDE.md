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
│   ├── lexer.rs       # Tokenizer
│   ├── parser.rs      # AST parser
│   ├── ast.rs         # AST definitions
│   ├── interpreter.rs # Tree-walking interpreter
│   └── stdlib.rs      # Standard library (ilo)
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
`li`, `e`, `la`, `open`, `pini`, `ilo`, `pali`, `pana`, `wile`, `taso`, `suli`, `lili`, `sama`, `jo`, `lon`, `ala`

### Core Syntax Patterns
- Assignment: `x li jo e Expr`
- Function call: `NAME e (args)`
- Function def: `ilo NAME li pali e (params) la open ... pini`
- If/else: `Cond la open ... pini taso open ... pini`
- While: `wile Cond la open ... pini`
- Return: `pana e Expr`
- Compare: `x li suli e y` (>), `x li lili e y` (<), `x li sama e y` (==)

### Types
- Number: `10`, `3.14`
- String: `"pona"`
- Boolean: `lon` (true), `ala` (false/null)

## Implementation Notes

- File extension: `.lipo`
- Use `nom` or `pest` for parsing
- Tree-walking interpreter for MVP
- All stdlib functions are regular `ilo` (no special syntax)

## Testing Strategy

1. Unit tests for lexer tokens
2. Unit tests for parser AST output
3. Integration tests running `.lipo` files
4. Test each stdlib function
