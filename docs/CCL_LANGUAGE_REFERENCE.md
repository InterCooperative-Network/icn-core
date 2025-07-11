# Cooperative Contract Language (CCL) Reference

This reference summarizes all syntax currently supported by the CCL compiler.
It is intended for cooperative developers writing governance policies and
economic logic for ICN nodes.

## Basic Concepts

CCL source files consist of function definitions, optional struct definitions,
and policy rules. Whitespace and `//` comments are ignored.

```ccl
// simple entry point
fn run() -> Integer { return 0; }
```

### Types

Built in primitive types:

- `Integer` – 64‑bit signed integer
- `Mana` – alias of `Integer` for economic values
- `Bool`  – boolean (`true` / `false`)
- `String` – UTF‑8 text stored in memory
- `Array<T>` – dynamic arrays
- `Option` and `Result` – for nullable values and error handling
- `Did` – decentralized identifier
- `Proposal` and `Vote` – governance primitives

User defined structs can bundle fields:

```ccl
struct Member { id: Did, name: String }
```

### Functions

Functions declare typed parameters and return a type:

```ccl
fn add(a: Integer, b: Integer) -> Integer {
    return a + b;
}
```

### Statements

Supported statements inside blocks:

- variable binding with `let`
- expression statements
- `return` expressions
- `if` / `else` conditional blocks
- `while` loops

### Expressions

Expressions support numeric and boolean operators, string concatenation, array
literals, and function calls. Arrays use helper functions such as `array_push`
and `array_len` for mutation.

Option and Result values use the variants `Some`, `None`, `Ok` and `Err`.
Pattern matching inspects these variants:

```ccl
fn divide(a: Integer, b: Integer) -> Result<Integer> {
    if b == 0 { return Err(1); }
    return Ok(a / b);
}

let result = divide(10, 2);
match result {
    Ok(v) => log_success(v),
    Err(e) => log_error(e),
}
```

### Policy Rules

Policy oriented contracts can declare rules which evaluate an expression and then
perform an action:

```ccl
rule charge_high when cost > 100 then charge cost
rule allow_basic when cost <= 100 then allow
```

Actions are `allow`, `deny`, or `charge <expr>`.

### Imports

External files can be imported with an alias:

```ccl
import "./common.ccl" as common;
```

### Entry Point

Contracts executed as mesh jobs expose a `run` function. Additional helper
functions may be defined as needed.

See `icn-ccl/examples/` for real‑world contract templates demonstrating the
language.
