#![allow(clippy::eval_order_dependence)]

use indexmap::IndexMap;
use syn::parse::{Parse, ParseStream, Result};

mod kw {
    syn::custom_keyword!(defaults);
}

pub struct Attributes {
    items: syn::punctuated::Punctuated<Attribute, syn::Token![,]>,
}

impl Attributes {
    pub fn defaults(&self) -> IndexMap<String, (proc_macro2::Span, syn::Expr)> {
        let mut map = IndexMap::new();
        for attribute in &self.items {
            let Attribute::Defaults(defaults) = attribute;
            for default in &defaults.defaults {
                map.insert(
                    default.name.to_string(),
                    (default.name.span(), default.value.clone()),
                );
            }
        }
        map
    }
}

pub enum Attribute {
    Defaults(Defaults),
}

impl Parse for Attributes {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            items: input.parse_terminated(Attribute::parse)?,
        })
    }
}

impl Parse for Attribute {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(kw::defaults) {
            input.parse().map(Self::Defaults)
        } else {
            Err(lookahead.error())
        }
    }
}

pub struct Default {
    name: syn::Ident,
    _eq_token: syn::Token![=],
    value: syn::Expr,
}

impl Parse for Default {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            name: input.parse()?,
            _eq_token: input.parse()?,
            value: input.parse()?,
        })
    }
}

pub struct Defaults {
    _keyword: kw::defaults,
    _bracket_token: syn::token::Paren,
    defaults: syn::punctuated::Punctuated<Default, syn::Token![,]>,
}

impl Parse for Defaults {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            _keyword: input.parse()?,
            _bracket_token: syn::parenthesized!(content in input),
            defaults: content.parse_terminated(Default::parse)?,
        })
    }
}
