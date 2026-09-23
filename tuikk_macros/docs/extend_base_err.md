# `#[extend_base_err]`

Attribute macro. Adds a shared `BaseErr` variant to a domain error enum, so each module doesn't have to write that variant by hand. Also derives `Debug` and `thiserror::Error` if they are missing.

[← Back to crate README](../README.md)

---

## Contents

- [Why](#why)
- [Requirements](#requirements)
- [Quick start](#quick-start)
- [What the macro generates](#what-the-macro-generates)
- [Arguments](#arguments)
- [Rules and requirements](#rules-and-requirements)
- [Compile errors](#compile-errors)
- [Known limitations](#known-limitations)
- [Tests](#tests)

---

## Why

In `tuikk`, every domain module (Docker, config, UI, …) has its own `thiserror` enum. Every one of these enums also needs to wrap the shared `BaseErr`, so that `?` works across module boundaries:

```rust
#[derive(Debug, thiserror::Error)]
pub enum DockerErr {
    #[error("Container not found: {0}")]
    NotFound(String),

    // Repeated in every domain enum:
    #[error(transparent)]
    Base(#[from] crate::tuikk_core::errors::BaseErr),
}
```

`#[extend_base_err]` generates that last variant, plus the `Debug` and `thiserror::Error` derives:

```rust
#[extend_base_err]
pub enum DockerErr {
    #[error("Container not found: {0}")]
    NotFound(String),
}
```

---

## Requirements

In the application crate's `Cargo.toml`:

```toml
[dependencies]
tuikk_macros = { path = "./tuikk_macros" }
thiserror = "2"   # required: the generated code refers to ::thiserror::Error
```

> A proc macro cannot bring dependencies along with it. The crate that **uses** `#[extend_base_err]` must list `thiserror` as a dependency itself.

---

## Quick start

```rust
use tuikk_macros::extend_base_err;

#[extend_base_err]
pub enum DockerErr {
    #[error("Container not found: {0}")]
    NotFound(String),

    #[error("Docker API error: {0}")]
    Api(#[from] bollard::errors::Error),
}

fn inspect(id: &str) -> Result<(), DockerErr> {
    do_base_work()?;            // Result<_, BaseErr> -> DockerErr::Base via From
    Err(DockerErr::NotFound(id.to_owned()))
}
```

---

## What the macro generates

Input:

```rust
#[extend_base_err]
#[derive(Clone)]
pub enum DockerErr {
    #[error("Container not found: {0}")]
    NotFound(String),
}
```

Expanded output (roughly what `cargo expand` shows):

```rust
#[derive(Debug)]
#[derive(::thiserror::Error)]
#[derive(Clone)]
pub enum DockerErr {
    #[error("Container not found: {0}")]
    NotFound(String),

    #[error(transparent)]
    Base(#[from] crate::tuikk_core::errors::BaseErr),
}
```

What the macro does:

| Step | Behavior |
| :--- | :--- |
| 1 | Checks that the item is an `enum`. Anything else is a compile error. |
| 2 | Refuses to run if the enum already has a variant named `Base`. |
| 3 | Scans the existing `#[derive(...)]` attributes for `Debug` and `Error`. |
| 4 | Appends `Base(#[from] <BaseErr path>)` with `#[error(transparent)]` as the **last** variant. |
| 5 | Adds `#[derive(Debug)]` only if `Debug` is not already derived. |
| 6 | Adds `#[derive(::thiserror::Error)]` only if `Error` is not already derived. |

Because the variant uses `#[error(transparent)]`, both `Display` and `source()` are forwarded to the inner `BaseErr`. The outer enum does not add any message of its own.

Because the variant uses `#[from]`, thiserror generates `impl From<BaseErr> for YourEnum`, so `?` converts `BaseErr` automatically.

---

## Arguments

| Usage | `BaseErr` path used |
| :--- | :--- |
| `#[extend_base_err]` | `crate::tuikk_core::errors::BaseErr` (default) |
| `#[extend_base_err(crate::errors::BaseErr)]` | the path you give |
| `#[extend_base_err(::shared::BaseErr)]` | a type from another crate |

The argument must be a valid Rust path (`syn::Path`). Anything else, such as a string literal or `key = value`, is a compile error.

`crate` in the default path refers to **the crate that calls the macro**, not `tuikk_macros`. If `BaseErr` lives somewhere else in your crate, pass the path explicitly.

---

## Rules and requirements

1. **Put `#[extend_base_err]` first, above every `#[derive(...)]`.**
   An attribute macro can only reliably see the attributes below it. If `#[derive(Debug)]` sits above the macro, the macro may not detect it and will add a second `Debug` derive, which gives `conflicting implementations of trait Debug`.

   ```rust
   // ✅
   #[extend_base_err]
   #[derive(Clone)]
   pub enum E { /* ... */ }

   // ❌
   #[derive(Debug)]
   #[extend_base_err]
   pub enum E { /* ... */ }
   ```

2. **Every variant you write needs an `#[error("...")]` attribute.** This is a thiserror rule: every variant of a derived `Error` enum must have a message, or be `transparent`.

3. **`BaseErr` must implement `std::error::Error`.** `#[error(transparent)]` requires it. If `BaseErr` is itself a `thiserror` enum, this is already the case.

4. **Don't declare a variant named `Base`.** That name is reserved for the generated variant.

5. **You don't need to derive `Debug` or `thiserror::Error` yourself.** If you do, the macro detects it and skips its own derive.

---

## Compile errors

| Situation | Message |
| :--- | :--- |
| Macro applied to a `struct`, `fn`, … | `#[extend_base_err] can only be applied to enums` (points at the item) |
| Enum already has a `Base` variant | `variant \`Base\` is generated by #[extend_base_err]; rename or remove it` (points at the variant) |
| Syntax error inside the enum | The original `syn` parse error, unchanged |
| Argument is not a path | `syn` parse error pointing at the argument |
| `thiserror` missing from the calling crate | `failed to resolve: could not find \`thiserror\`` |
| `BaseErr` path does not exist | `cannot find type \`BaseErr\`` — pass the right path as an argument |

---

## Known limitations

- **`Error` detection only looks at the last path segment.** `#[derive(Error)]`, `#[derive(thiserror::Error)]` and `#[derive(some_other_crate::Error)]` are all treated as "already derived". Using an `Error` derive from another crate together with this macro will fail on the `#[error]` / `#[from]` attributes.
- **No check for a second `#[from] BaseErr`.** If another variant already has `#[from] BaseErr`, thiserror reports a duplicate `From` impl.
- **The generated variant name is fixed to `Base`.**
- **The generated variant is always last.** This only matters if you depend on variant order, for example with `#[repr]` discriminants.

---

## Tests

Unit tests live in `src/base_err_macro.rs`. Run them with `cargo test -p tuikk_macros`.

They cover:

- adds the `Base` variant and the missing derives
- does not duplicate existing `Debug` / `Error` derives
- respects a custom `BaseErr` path
- rejects an existing `Base` variant
- rejects non-enum items
- keeps the original syntax error for malformed enums
