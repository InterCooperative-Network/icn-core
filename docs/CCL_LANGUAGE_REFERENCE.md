# Cooperative Contract Language (CCL) Reference

This document describes the syntax currently supported by the CCL compiler in `icn-core`.
It complements the `icn-ccl` crate documentation and example contracts.

## 1. File Structure

A CCL source file is a **policy** composed of function definitions and policy statements.
A policy must define at least one `fn run()` entry point if it is intended for execution.

```
fn helper(arg: Integer) -> Integer {
    return arg + 1;
}

rule allow_all when true then allow

fn run() -> Integer {
    return helper(5);
}
```

## 2. Functions

Functions are declared using `fn` and may return any supported type.
Parameters require type annotations.

```
fn compute_cost(cores: Integer, memory: Integer) -> Mana {
    let base = cores * 10 + memory / 2;
    return base;
}
```

## 3. Policy Statements

Two kinds of policy statements are supported:

- **`rule` definitions** – declarative permissions executed by the runtime.
- **`import` statements** – include another policy file under an alias.

Example rule:

```
rule charge_big_jobs when cores > 8 then charge compute_cost(cores, memory)
```

Example import:

```
import "common.ccl" as common;
```

## 4. Statements

Inside function blocks you may use:

- `let` variable declarations
- expression statements ending with `;`
- `return` statements
- `if`/`else` conditional blocks
- `while` loops

```
let total = 0;
while count > 0 {
    total = total + count;
    count = count - 1;
}
if total > 100 {
    return 1;
} else {
    return 0;
}
```

## 5. Expressions

CCL supports integer, boolean and string literals, arrays, identifiers and function calls.
Operators follow Rust-like precedence with `!`, `*`, `/`, `+`, `-`, comparisons, `&&` and `||`.

Arrays are written `[expr, expr]` and can be indexed with `array[index]`.

## 6. Types

Available primitive types are:

- `Integer`
- `Mana` (alias of `Integer` used for accounting)
- `Bool`
- `String`
- `Did` for decentralized identifiers
- `Array<T>` for arrays of any type
- `Proposal` and `Vote` for governance helpers

## 7. Actions

`rule` definitions end with an action:

- `allow` – permit the operation
- `deny` – deny the operation
- `charge EXPR` – deduct the mana returned by `EXPR`

Actions are evaluated when `condition` in `when` is true.

## 8. Comments

Line comments start with `//` and run to the end of the line.

## 9. Entry Point

The runtime expects a `fn run()` function. Its return value is propagated back to
callers or used by the job scheduler. Complex policies may define additional helper
functions and rules.

## 10. Further Reading

See `icn-ccl/examples/` for real-world policies and `icn-ccl/README.md` for
compilation instructions.
