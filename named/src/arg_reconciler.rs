use crate::attr_parser::Attributes;
use indexmap::IndexMap;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::ToTokens;
use std::collections::BTreeSet;
use syn::{FnArg, ItemFn, Pat};

pub struct ArgDetails {
    pub args: Vec<Ident>,
    pub defaults: IndexMap<String, Option<proc_macro2::TokenStream>>,
}

pub fn reconcile(f: &ItemFn, attr: TokenStream) -> syn::Result<ArgDetails> {
    let args: Result<Vec<_>, _> = f
        .sig
        .inputs
        .iter()
        .map(|arg| match arg {
            FnArg::Receiver(_) => {
                Err(syn::Error::new_spanned(arg, "`named` does not currently support functions which take `self`."))
            },
            FnArg::Typed(pat_type) => {
                if let Pat::Ident(ident) = pat_type.pat.as_ref() {
                    Ok(ident.ident.clone())
                } else {
                    panic!("Didn't recognise function signature - expected all args to be idents, but found: {:?}", pat_type);
                }
            }
        })
        .collect();
    let args = args?;

    let attr: Attributes = syn::parse_macro_input::parse(attr)?;
    let defaults = attr.defaults();

    let fn_arg_names = args
        .iter()
        .map(|ident| ident.to_string())
        .collect::<BTreeSet<_>>();
    let attr_arg_names = defaults
        .keys()
        .map(|ident| ident.to_string())
        .collect::<BTreeSet<_>>();
    let extras = attr_arg_names
        .difference(&fn_arg_names)
        .cloned()
        .collect::<Vec<_>>();
    if !extras.is_empty() {
        let extras_plural_suffix;
        let span;
        let extras_str;
        if extras.len() == 1 {
            extras_plural_suffix = "";
            span = defaults[&extras[0]].0;
            extras_str = format!("`{}`", &extras[0]);
        } else {
            extras_plural_suffix = "s";
            span = Span::call_site();
            extras_str = format!("[{}]", extras.join(", "));
        }
        return Err(syn::Error::new(
            span,
            format!(
                "Unrecognized argument{} - attribute had argument{} {} but function takes argument{}: [{}]",
                extras_plural_suffix,
                extras_plural_suffix,
                extras_str,
                if fn_arg_names.len() == 1 { "" } else { "s" },
                fn_arg_names.into_iter().collect::<Vec<_>>().join(", "),
            )
        ));
    }

    let defaults = fn_arg_names
        .into_iter()
        .map(|arg| {
            let value = defaults
                .get(&arg)
                .map(|(_span, value)| value.to_token_stream());
            (arg, value)
        })
        .collect();

    Ok(ArgDetails { args, defaults })
}
