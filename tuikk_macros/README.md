# tuikk_macros

Procedural macros for the `tuikk` workspace.

This README covers the crate as a whole: which macros exist, how the crate is organized, and how to add a new macro. Usage details for each macro live in its own file under [`docs/`](docs/).

---

## Table of contents

- [Macros](#macros)
- [Installation](#installation)
- [Crate layout](#crate-layout)
- [Proc-macro crate rules](#proc-macro-crate-rules)
- [Adding a new macro](#adding-a-new-macro)
- [Development](#development)

---

## Macros

| Macro | Kind | Summary | Docs |
| :--- | :--- | :--- | :--- |
| `#[extend_base_err]` | Attribute | Adds a `Base(#[from] BaseErr)` variant to a domain error enum and derives `Debug` + `thiserror::Error` if missing | [docs/extend_base_err.md](docs/extend_base_err.md) |

> When you add a macro, add a row here. Keep the summary to one line; everything else goes in the macro's own doc file.

---

## Installation

In the application crate's `Cargo.toml`:

```toml
[dependencies]
tuikk_macros = { path = "./tuikk_macros" }
```

A proc macro only emits code; it cannot bring dependencies along. If the generated code refers to another crate (for example `::thiserror`), the **calling** crate must depend on that crate itself. Each macro's doc file lists what it needs under **Requirements**.

---

## Crate layout

```
tuikk_macros/
├── Cargo.toml              # [lib] proc-macro = true
├── README.md               # this file
├── docs/
│   ├── _template.md        # starting point for a new macro's docs
│   └── extend_base_err.md
└── src/
    ├── lib.rs              # crate root: thin #[proc_macro_*] wrappers only
    └── base_err_macro.rs   # logic for #[extend_base_err] + unit tests
```

`Cargo.toml`:

```toml
[lib]
proc-macro = true

[dependencies]
syn = { version = "3", features = ["full"] }
quote = "1"
proc-macro2 = "1"
```

### Two layers

| Layer | File | Token type | Job |
| :--- | :--- | :--- | :--- |
| Wrapper | `src/lib.rs` | `proc_macro::TokenStream` | Registers the macro with `rustc`, converts types with `.into()`, turns `syn::Error` into `compile_error!` |
| Logic | `src/<module>.rs` | `proc_macro2::TokenStream` | Parses input, validates it, generates code. Returns `syn::Result<TokenStream>` |

`proc_macro::TokenStream` only works while `rustc` is expanding a macro, so code that uses it cannot be unit-tested. Keeping all logic on `proc_macro2` means every module can be tested with plain `cargo test`.

Wrapper example in `lib.rs`:

```rust
use proc_macro::TokenStream;

mod base_err_macro;

/// Adds `Base(#[from] BaseErr)` and derives `Debug` + `thiserror::Error` if missing.
/// See docs/extend_base_err.md.
#[proc_macro_attribute]
pub fn extend_base_err(args: TokenStream, item: TokenStream) -> TokenStream {
    base_err_macro::expand(args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
```

---

## Proc-macro crate rules

| Rule | What happens if you break it |
| :--- | :--- |
| Functions tagged `#[proc_macro]`, `#[proc_macro_attribute]` or `#[proc_macro_derive]` must be **defined in `lib.rs`**. Re-exporting with `pub use` does not work. | `functions tagged with #[proc_macro_attribute] must currently reside in the root of the crate` |
| The crate can only export macros. Declare modules with `mod` (not `pub mod`) and helpers with `pub(crate)`. | `proc-macro crate types currently cannot export any items other than functions tagged with #[proc_macro]...` |
| Only `lib.rs` uses `proc_macro::TokenStream`. Logic modules import `proc_macro2::TokenStream`. | `mismatched types: TokenStream and proc_macro2::TokenStream have similar names, but are actually distinct types` |
| Use `syn::parse2::<T>(tokens)?` in logic modules, not `parse_macro_input!`. | `parse_macro_input!` only accepts `proc_macro::TokenStream` |
| Generated code uses absolute paths (`::thiserror::Error`, `::core::fmt::Debug`). | A module in the calling crate with the same name would shadow the path |
| Types or traits that the app needs at runtime go in a normal crate (for example `tuikk_common`), not here. | They can't be exported from a proc-macro crate |
| Don't name a module `core`. | Clashes with the built-in `::core` crate |

---

## Adding a new macro

### 1. Create the logic module

`src/<module>.rs`. The signature depends on the macro kind:

| Kind | Called as | Signature of `expand` |
| :--- | :--- | :--- |
| Attribute | `#[my_attr(args)]` | `pub(crate) fn expand(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream>` |
| Derive | `#[derive(MyTrait)]` | `pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream>` |
| Function-like | `my_macro!(...)` | `pub(crate) fn expand(input: TokenStream) -> syn::Result<TokenStream>` |

Template:

```rust
use proc_macro2::TokenStream;
use quote::quote;
use syn::Item;

pub(crate) fn expand(args: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    // 1. Parse args (reject unknown input instead of ignoring it)
    // 2. Parse the item; keep the original syn error, add your own only for wrong item kinds
    let item: Item = syn::parse2(item)?;

    // 3. Validate; return syn::Error with a span that points at the problem
    // 4. Generate code
    Ok(quote! { #item })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn happy_path() {
        let out = expand(quote!(), quote! { fn foo() {} }).unwrap().to_string();
        assert!(out.contains("foo"));
    }

    #[test]
    fn rejects_invalid_input() {
        assert!(expand(quote!(), quote! { struct S; }).is_err());
    }
}
```

### 2. Register it in `lib.rs`

```rust
mod my_module;

/// One-line summary. See docs/my_macro.md.
#[proc_macro_attribute]
pub fn my_macro(args: TokenStream, item: TokenStream) -> TokenStream {
    my_module::expand(args.into(), item.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
```

For a derive macro, declare helper attributes on the wrapper:

```rust
#[proc_macro_derive(MyTrait, attributes(my_helper))]
pub fn derive_my_trait(input: TokenStream) -> TokenStream {
    my_module::expand(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
```

> Once there are several macros, a `macro_rules!` helper in `lib.rs` can generate the `mod` line and wrapper for each one. The code it generates still sits at the crate root, so `rustc` accepts it.

### 3. Write the docs

1. Copy [`docs/_template.md`](docs/_template.md) to `docs/<macro_name>.md` and fill it in.
2. Add a row to the [Macros](#macros) table above.
3. Keep the doc comment on the wrapper short and point it at the doc file. That comment is what users see on hover.

### Checklist

- [ ] Logic in `src/<module>.rs`, on `proc_macro2` only, returning `syn::Result`
- [ ] Wrapper in `lib.rs`, module declared with `mod`
- [ ] Unit tests for the happy path and each error case
- [ ] Errors point at the right span and don't swallow the original `syn` error
- [ ] Generated code uses absolute paths
- [ ] `docs/<macro_name>.md` written from the template
- [ ] Row added to the Macros table

---

## Development

```bash
# Unit tests for all macro logic
cargo test -p tuikk_macros

# See what a macro expands to (needs: cargo install cargo-expand)
cargo expand -p tuikk path::to::module

# Lints
cargo clippy -p tuikk_macros -- -W unreachable_pub
```