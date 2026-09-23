use proc_macro::TokenStream;
mod base_err_macro;

#[cfg(test)]
mod tests;
#[proc_macro_attribute]
pub fn extend_base_err(_args: TokenStream, input: TokenStream) -> TokenStream {
    base_err_macro::extend_base_err(_args.into(), input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
