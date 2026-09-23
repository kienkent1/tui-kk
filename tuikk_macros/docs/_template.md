# `<macro_name>`

<Kind: attribute / derive / function-like> macro. <One or two sentences on what it does.>

[← Back to crate README](../README.md)

---

## Contents

- [Why](#why)
- [Requirements](#requirements)
- [Quick start](#quick-start)
- [What the macro generates](#what-the-macro-generates)
- [Arguments](#arguments)
- [Rules](#rules)
- [Compile errors](#compile-errors)
- [Known limitations](#known-limitations)
- [Tests](#tests)

---

## Why

<The boilerplate this macro removes. Show the hand-written version first, then the version using the macro.>

```rust
// before
```

```rust
// after
```

---

## Requirements

<Crates the calling crate must depend on because the generated code refers to them. Write "None" if there aren't any.>

```toml
[dependencies]
tuikk_macros = { path = "./tuikk_macros" }
# other_crate = "x"   # required: generated code refers to ::other_crate
```

---

## Quick start

```rust
use tuikk_macros::<macro_name>;

// minimal, realistic example
```

---

## What the macro generates

Input:

```rust
```

Expanded output (roughly what `cargo expand` shows):

```rust
```

| Step | Behavior |
| :--- | :--- |
| 1 | |

---

## Arguments

| Usage | Effect |
| :--- | :--- |
| `<call with no args>` | default |

<Say what happens with invalid arguments.>

---

## Rules

1. <Attribute ordering, required attributes on fields or variants, reserved names, …>

---

## Compile errors

| Situation | Message |
| :--- | :--- |
| | |

---

## Known limitations

- <Anything the macro doesn't check, or behavior that might surprise users.>

---

## Tests

Unit tests live in `src/<module>.rs`. Run them with `cargo test -p tuikk_macros`.

They cover:

- happy path
- <each error case>
