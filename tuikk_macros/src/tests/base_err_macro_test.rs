use super::*;
use crate::base_err_macro::extend_base_err;
use proc_macro2::TokenStream;
use quote::quote;
fn run(args: TokenStream, input: TokenStream) -> String {
    extend_base_err(args, input).unwrap().to_string()
}

#[test]
fn adds_base_variant_and_derives() {
    let out = run(
        quote!(),
        quote! {
            pub enum DockerErr {
                #[error("not found: {0}")]
                NotFound(String),
            }
        },
    );
    assert!(out.contains("Base"));
    assert!(out.contains(":: thiserror :: Error"));
    assert!(out.contains("derive (Debug)"));
    assert!(out.contains("crate :: tuikk_core :: errors :: BaseErr"));
}

#[test]
fn does_not_duplicate_existing_derives() {
    let out = run(
        quote!(),
        quote! {
            #[derive(Debug, thiserror::Error)]
            enum E { #[error("x")] X }
        },
    );
    assert_eq!(out.matches("Debug").count(), 1);
    assert_eq!(out.matches("Error").count(), 1);
}

#[test]
fn custom_base_path() {
    let out = run(quote!(crate::errors::MyBase), quote! { enum E {} });
    assert!(out.contains("crate :: errors :: MyBase"));
}

#[test]
fn rejects_duplicate_base() {
    assert!(extend_base_err(quote!(), quote! { enum E { Base(u8) } }).is_err());
}

#[test]
fn rejects_non_enum() {
    let err = extend_base_err(quote!(), quote! { struct S; }).unwrap_err();
    assert!(err.to_string().contains("can only be applied to enums"));
}

#[test]
fn keeps_original_syntax_error() {
    // Enum có lỗi cú pháp: lỗi gốc được giữ, không bị thay bằng "only enums"
    let err = extend_base_err(quote!(), quote! { enum E { A B } }).unwrap_err();
    assert!(!err.to_string().contains("can only be applied to enums"));
}
