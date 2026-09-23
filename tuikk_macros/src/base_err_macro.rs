use proc_macro2::TokenStream;
use quote::quote;
use syn::{Item, Path};

pub(crate) fn extend_base_err(args: TokenStream, input: TokenStream) -> syn::Result<TokenStream> {
    let base_path: Path = if args.is_empty() {
        syn::parse_quote!(crate::tuikk_core::errors::BaseErr)
    } else {
        syn::parse2(args)?
    };
    let mut item_enum = match syn::parse2::<Item>(input)? {
        Item::Enum(e) => e,
        other => {
            return Err(syn::Error::new_spanned(
                other,
                "#[extend_base_err] can only be applied to enums",
            ))
        }
    };

    // check if enum has varant Base
    if let Some(v) = item_enum.variants.iter().find(|v| v.ident == "Base") {
        return Err(syn::Error::new(
            v.ident.span(),
            "The Base variant has already been declared; rename it or remove #[extend_base_err].",
        ));
    }

    let (mut has_debug, mut has_err) = (false, false);

    for attr in item_enum
        .attrs
        .iter()
        .filter(|a| a.path().is_ident("derive"))
    {
        attr.parse_nested_meta(|meta| {
            let last = meta.path.segments.last().map(|s| s.ident.to_string());

            match last.as_deref() {
                Some("Debug") => has_debug = true,
                Some("Error") => has_err = true,
                _ => {}
            }

            Ok(())
        })?;
    }

    item_enum.variants.push(syn::parse_quote! {
        #[error(transparent)]
        Base(#[from] #base_path)
    });

    let debug = (!has_debug).then(|| quote!(#[derive(Debug)]));
    let err = (!has_err).then(|| quote! (#[derive(::thiserror::Error)]));
    Ok(quote! {
        #debug
        #err
        #item_enum
    })
}
