# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Lipona is a minimal programming language whose syntax is modeled on Toki Pona grammar, implemented as a tree-walking interpreter in Rust. The guiding philosophy is to keep syntax minimal and extend functionality through functions (`ilo`), not new syntax — so when adding a feature, prefer a stdlib function over a grammar change.

`Lipona.md` is the authoritative language specification. README.md (written in Toki Pona) shows user-facing syntax examples.

## Development Commands

The repo ships a Nix flake dev shell. Enter it with `nix develop` (or use direnv, which is pre-configured).

```bash
cargo build                 # build
cargo run -- <file.lipo>    # run a .lipo file
cargo run -- -e '<code>'    # run an inline snippet
cargo test                  # run all tests
cargo test <name>           # run a single test (matches test fn name substring)
cargo clippy                # lint
cargo fmt                   # format
cargo watch -x run          # auto-rebuild
```

Slash-command shortcuts also exist: `/test`, `/run`, `/fmt`, `/clippy`, `/check`, `/spec`.

## Architecture

The pipeline is: source → `pest` PEG parse → AST → tree-walking interpreter. Four core modules in `src/`:

- **`lipona.pest`** — PEG grammar. Expression precedence is encoded by nested rules (`comparison` → `add_expr` → `mul_expr` → `unary_expr` → `primary`). Comparison operators are tried longest-first (`suli_sama` before `suli`) to avoid prefix ambiguity. String literals (`${ ... }$` in pest) have two alternating parts: `string_literal` and `interpolation` (`{expr}`) — this is what supports template strings.
- **`parser.rs`** — Converts pest pairs to the AST in `ast.rs`. Receives spans from pest for error reporting.
- **`ast.rs`** — `Expr`, `Stmt`, `BinOp`, `StringPart`. A template string is `Expr::TemplateString(Vec<StringPart>)` where each part is either a `Literal(String)` or `Interpolation(Box<Expr>)`.
- **`interpreter.rs`** — `Interpreter` holds `Environment` + `StdLib` + `call_depth`. `Environment` is a `Vec<HashMap>` scope stack.
- **`stdlib.rs`** — Built-in `ilo` functions. Checked before user-defined functions in `call_function_inner`, so stdlib names effectively shadow user definitions.

### Runtime value model — non-obvious

`lon` (true) and `ala` (false/null) are represented as **two distinct `Value` variants**, not a single `Value::Bool(bool)`:

- `Value::Bool` (unit — only represents `lon`/true)
- `Value::Ala` (represents both false and null/absent)

Comparison operators return `Value::Bool` for true, `Value::Ala` for false. `Value::is_truthy()` treats `Ala`, `0`, `""`, empty list/map as falsy. Keep this in mind when touching the interpreter — conflating these will break conditionals.

### Closure environment snapshot

`Value::Function` carries a `captured: Vec<HashMap<String, Value>>` field — a snapshot of the scope stack taken when the function value is created (for named `FuncDef` and anonymous `Expr::Lambda` alike). On call, the interpreter swaps `captured` into the current scope stack, pushes a fresh inner scope for the parameters, runs the body, and restores the caller's scopes via `Environment::replace_scopes()`.

Two subtleties:
- **Recursion** — a `FuncDef` is first `define`d with `Value::Ala` under its name, *then* the captured snapshot is taken (so the name is already in scope), *then* the real `Value::Function` replaces the placeholder. The function sees itself through the captured env.
- **Live globals for recursion / top-level mutation** — on every call, the captured snapshot's global scope (index 0) is refreshed from the caller's current globals before executing. So later-defined top-level bindings (including the callee itself after its `Value::Function` is bound) are visible, and a function that was originally captured before its siblings existed can still call them.

Stdlib calls do **not** swap scopes — they operate on evaluated argument values only.

### Safety limits

Hardcoded in `interpreter.rs`: `MAX_LOOP_ITERATIONS = 10_000_000`, `MAX_CALL_DEPTH = 1000`. Exceeding either raises `pakala: InfiniteLoop` / `StackOverflow`. All runtime errors have the `pakala:` prefix via `thiserror`.

## Language Reference (quick)

- Reserved keywords: `la`, `open`, `pini`, `ilo`, `pana`, `wile`, `taso`, `suli`, `lili`, `suli_sama`, `lili_sama`, `sama`, `jo`, `lon`, `ala`
- Assignment: `x jo Expr` — note `jo` is the assignment operator, not `=`
- If/else: `Cond la open ... pini taso open ... pini` (the `taso` block is optional)
- While: `wile Cond la open ... pini`
- Function def: `ilo NAME (params) open ... pini`; return: `pana Expr`; implicit return is `ala`
- Lambda (anonymous function expression): `ilo (params) open ... pini` — evaluates to a callable `Value::Function`. Bind it with `f jo ilo (...) open ... pini`, pass it as an argument, or return it. Calls still require an identifier callee: `f(a, b)` (not `(expr)(a, b)`).
- Comparisons: `suli` (>), `lili` (<), `suli_sama` (>=), `lili_sama` (<=), `sama` (==). No `!=`.
- Template strings: `"Hello, {name}!"` — `{...}` interpolates any expression. Escapes: `\n \t \r \\ \" \{ \}`
- Types: Number (f64), String, `lon`, `ala`, kulupu (list), nasin (map), ilo (function)
- Identifiers are ASCII only (`[a-zA-Z_][a-zA-Z0-9_]*`); names may be Toki Pona or English

## Testing Strategy

There is no `tests/` directory — integration testing is done by running `.lipo` files from `examples/` (see `test_all.lipo` for a battery). Unit tests live inline in each `src/*.rs` module under `#[cfg(test)]`.

When adding a language feature, add (a) a grammar rule in `lipona.pest`, (b) AST construction in `parser.rs`, (c) evaluation in `interpreter.rs`, and (d) an example `.lipo` file exercising it. When adding a stdlib function, it is a pure addition to `stdlib.rs` — no grammar or interpreter changes needed, since all stdlib calls go through the generic `FuncCall` path. This is the core design invariant: **new capabilities should be new `ilo`, not new syntax.**

## Error Semantics (from Lipona.md §8)

- Undefined variable/function, 0 division, type mismatch, out-of-bounds index, wrong arity → `pakala` (runtime error, aborts)
- Missing map key on read → returns `ala` (does not raise)
- Missing map key on write → `pakala`
