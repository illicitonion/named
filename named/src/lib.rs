use indexmap::map::IndexMap;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;
use syn::{ItemFn, Token};

mod arg_reconciler;
mod attr_parser;

/// This procedural macro allows you to produce functions which can be called with named arguments, optionally with default values. The function must be called as a macro, rather than like a "real" function.
///
/// > ⚠️ **Warning:** This crate is intended as an experiment to explore potential ways to provide named arguments in Rust - while it _should_ work, I wouldn't necessarily encourage its use. In particular, it has significant limitations (such as not supporting functions inside `impl` blocks), and no real intention to work around the current language restrictions in order to remove them.
///
/// ```rust
/// use named::named;
///
/// #[named(defaults(a = false, b = false))]
/// fn or(a: bool, b: bool) -> bool {
///     a || b
/// }
///
/// fn main() {
///     // You can use defaults for everything:
///     assert!(!or!());
///
///     // Or just for some values:
///     assert!(or!(a = true));
///     assert!(or!(b = true));
///     assert!(!or!(a = false));
///     assert!(!or!(b = false));
///
///     // Or explicitly specify them all:
///     assert!(or!(a = true, b = false));
///     assert!(or!(a = false, b = true));
///     assert!(or!(a = true, b = true));
///     assert!(!or!(a = false, b = false));
/// }
/// ```
///
/// Arguments must be specified in the same order as they were declared in the function, so if you defined your function `fn or(a: bool, b: bool)` you couldn't call it `or!(b = true, a = true)`.
///
/// All arguments must be supplied with names, you can't mix and match, i.e. you can't call `or!(a = true, false)`.
///
/// Not all arguments need default values; you could do this:
/// ```rust
/// use named::named;
///
/// #[named(defaults(b = false))]
/// fn or(a: bool, b: bool) -> bool {
///     a || b
/// }
///
/// fn main() {
///     assert!(or!(a = true));
///     assert!(or!(a = true, b = true));
/// }
/// ```
///
/// Any const expression can be used as a default value:
/// ```rust
/// use named::named;
///
/// pub struct D {
///     pub value: u8,
/// }
///
/// const DEFAULT: D = D { value: 1 };
///
/// #[named(defaults(a = DEFAULT.value))]
/// fn is_one(a: u8) -> bool {
///     a == 1
/// }
///
/// fn main() {
///     assert!(is_one!());
/// }
/// ```
///
/// All of the smarts happen at compile time, so at runtime this macro results in plain function calls with no extra overhead.
///
/// Unfortunately, this can't currently be used for functions defined in `impl` blocks, e.g. those which take a `self` parameter. It's possible that [postfix macros](https://github.com/rust-lang/rfcs/pull/2442) could enable this nicely.
#[proc_macro_attribute]
pub fn named(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut f: ItemFn = syn::parse_macro_input!(item);

    // Name of the original function - we'll use this as our macro name.
    let name = f.sig.ident.clone();

    let arg_reconciler::ArgDetails { args, defaults } = match arg_reconciler::reconcile(&f, attr) {
        Ok(v) => v,
        Err(err) => {
            // Create a macro, so that the only error we get is about the ill-called proc_macro,
            // and the compiler doesn't also produce errors for every call-site about the macro not existing.
            let mut m = quote! { macro_rules! #name { ($($idents:ident = $exprs:expr),*) => { unimplemented!() } } };
            m.extend(err.to_compile_error());
            return m.into();
        }
    };

    // Name of the actual function we'll generate with one arg per arg of f.
    // This is considered a private implementation detail, and should not be relied on - it may change or be removed in a patch release.
    let dunder_name = syn::Ident::new(&format!("__{}", name), name.span());
    // Name of the inner macro we'll generate which accumulates non-named arguments from the front.
    // This is considered a private implementation detail, and should not be relied on - it may change or be removed in a patch release.
    let inner_name = syn::Ident::new(&format!("{}_inner", dunder_name), name.span());

    f.sig.ident = dunder_name.clone();

    let mut ts = f.into_token_stream();

    // Generate the inner macro which accumulates already-parsed-values with 0 or more named values.
    {
        let mut branches = Vec::with_capacity(5 * args.len() + 3);
        for completed in 0..=args.len() {
            let (already_parsed_exprs, still_being_parsed) = args.split_at(completed);

            let match_exprs: Punctuated<_, Token![,]> = already_parsed_exprs
                .iter()
                .map(|expr| quote! { $#expr:expr }.into_iter().collect::<TokenStream>())
                .collect();

            let already_parsed_exprs: Punctuated<_, Token![,]> = already_parsed_exprs
                .iter()
                .map(|expr| quote! { $#expr })
                .collect();

            let remaining_defaults: IndexMap<_, _> = defaults
                .iter()
                .skip(already_parsed_exprs.len())
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect();

            // First n are set, use defaults for the rest.
            branches.push({
                let missing_required: Vec<String> = remaining_defaults
                    .iter()
                    .filter(|(_k, v)| v.is_none())
                    .map(|(k, _v)| k.clone())
                    .collect();
                // No values have been provided, so any missing defaults is fatal.
                let rhs = if !missing_required.is_empty() {
                    report_missing(&missing_required)
                } else {
                    let mut values = already_parsed_exprs.clone();
                    values.extend(
                        remaining_defaults
                            .values()
                            .cloned()
                            // Unwrap OK - checked in the filter above.
                            .map(|v| v.unwrap())
                            .collect::<Punctuated<_, Token![,]>>(),
                    );
                    quote! { #dunder_name(#values) }
                };
                quote! { (#match_exprs) => { #rhs }; }
            });

            if let Some(next_missing_ident) = still_being_parsed.iter().next() {
                // First n are set, next is n+1, no more after.
                branches.push({
                    let mut match_exprs = match_exprs.clone();
                    match_exprs.push(quote! { #next_missing_ident = $#next_missing_ident:expr});
                    let mut values = already_parsed_exprs.clone();
                    values.push(quote! { $#next_missing_ident });
                    quote! { (#match_exprs) => { #inner_name!(#values) }; }
                });

                // Handle first n are set, next is n+1, more after.
                branches.push({
                    let mut match_exprs = match_exprs.clone();
                    match_exprs.push(quote! { #next_missing_ident = $#next_missing_ident:expr});
                    match_exprs.push(quote! { $($keys:ident = $values:expr),+ }.to_token_stream());
                    let mut exprs = already_parsed_exprs.clone();
                    exprs.push(quote! { $#next_missing_ident });
                    exprs.push(quote! { $($keys = $values),+ }.to_token_stream());
                    quote! { (#match_exprs) => { #inner_name!(#exprs) }; }
                });

                // Handle first n are set, next is not n+1, no more after.
                branches.push({
                    let mut match_exprs = match_exprs.clone();
                    match_exprs.push(quote! { $key:ident = $value:expr });
                    let mut values = already_parsed_exprs.clone();
                    // Unwrap OK: Our we know defaults is the same size as our loop iteration.
                    let rhs = match remaining_defaults.iter().next().unwrap() {
                        (_name, Some(next_default_value)) => {
                            values.push(quote! { #next_default_value });
                            values.push(quote! { $key = $value });
                            quote! { #inner_name!(#values) }
                        }
                        (missing_name, None) => {
                            // TODO: Would ideally specify all missing, not just next.
                            report_missing(&[missing_name.clone()])
                        }
                    };
                    quote! { (#match_exprs) => { #rhs }; }
                });

                // Handle first n are set, next is not n+1, more after.
                branches.push({
                    let mut match_exprs = match_exprs.clone();
                    match_exprs.push(quote! { $($keys:ident = $values:expr),+ });
                    let mut already_parsed_exprs = already_parsed_exprs.clone();
                    // Unwrap OK: Our we know defaults is the same size as our loop iteration.
                    let rhs = match remaining_defaults.iter().next().clone().unwrap() {
                        (_name, Some(next_default_value)) => {
                            already_parsed_exprs.push(quote! { #next_default_value });
                            already_parsed_exprs.push(quote! { $($keys = $values),+ });
                            quote! { #inner_name!(#already_parsed_exprs) }
                        }
                        (missing_name, None) => {
                            // TODO: Would ideally specify all missing, not just next.
                            report_missing(&[missing_name.clone()])
                        }
                    };
                    quote! { (#match_exprs) => { #rhs }; }
                });
            }
        }

        // All args given, yet we have one more!
        branches.push({
            let match_exprs: Punctuated<_, Token![,]> = args
                .iter()
                .map(|expr| quote! { $#expr:expr }.into_iter().collect::<TokenStream>())
                .collect();
            let expected_names = format_names(&args.iter().map(|v| v.to_string()).collect::<Vec<_>>());
            let expected_names = quote! { #expected_names };
            quote! { (#match_exprs, $ident:ident = $expr:expr) => { compile_error!(concat!("Unrecognized named argument - got value for argument `", stringify!($ident), "` but only expected ", #expected_names)) }; }
        });

        // All args given, yet we have more than one more!
        branches.push({
            let match_exprs: Punctuated<_, Token![,]> = args
                .iter()
                .map(|expr| quote! { $#expr:expr }.into_iter().collect::<TokenStream>())
                .collect();
            let expected_names = format_names(&args.iter().map(|v| v.to_string()).collect::<Vec<_>>());
            let expected_names = quote! { #expected_names };
            // TODO: Maybe mention all, not just first.
            quote! { (#match_exprs, $ident:ident = $expr:expr, $($idents:ident = $exprs:expr),+) => { compile_error!(concat!("Unrecognized named argument - got value for argument `", stringify!($ident), "` but only expected ", #expected_names)) }; }
        });

        ts.extend(quote! { macro_rules! #inner_name { #(#branches)* } });
    }

    // Generate the actual named-values macro, which only expects name-value pairs.
    {
        let mut branches = Vec::with_capacity(5);
        if args.is_empty() {
            branches.push(quote! { () => { #dunder_name() }; });
        } else {
            let first_name = args[0].clone();
            let first_expr = quote! { $#first_name:expr };

            let first_default = defaults.iter().next().map(|(_k, v)| v.clone()).unwrap();
            let first_default = first_default.map(|v| quote! { #v });

            branches
                .push(quote! { (#first_name = #first_expr) => { #inner_name!($#first_name) }; });
            branches.push(
                quote! { (#first_name = #first_expr, $($keys:ident = $values:expr),+) => { #inner_name!($#first_name, $($keys = $values),+) }; }
            );
            branches.push({
                let rhs = if first_default.is_some() {
                    quote! { #inner_name!(#first_default, $other = $other_value, $($keys = $values),+) }
                } else {
                    // TODO: Would ideally specify all missing, not just next.
                    report_missing(&[first_name.to_string()])
                };
                quote! { ($other:ident = $other_value:expr, $($keys:ident = $values:expr),+) => { #rhs }; }
            });
            branches.push({
                let rhs = if first_default.is_some() {
                    quote! { #inner_name!(#first_default, $other = $other_value) }
                } else {
                    report_missing(&[first_name.to_string()])
                };
                quote! { ($other:ident = $other_value:expr) => { #rhs }; }
            });
            branches.push(quote! { () => { #inner_name!() }; });
        }

        ts.extend(quote! {
            // foo fills in defaults until it finds its first :ident = :expr.
            // It is not allowed any bare :exprs.
            macro_rules! #name {
                #(#branches)*
            }
        });
    }
    ts.into()
}

fn report_missing(missing: &[String]) -> TokenStream {
    let maybe_s = if missing.len() == 1 { "" } else { "s" };
    let missing_str = format!(
        "Must specify value{} for non-defaulted argument{}: {}",
        maybe_s,
        maybe_s,
        format_names(missing),
    );
    quote! { compile_error!(#missing_str) }
}

fn format_names(names: &[String]) -> String {
    if names.len() == 1 {
        format!("`{}`", names[0])
    } else {
        format!("[{}]", names.join(", "))
    }
}
