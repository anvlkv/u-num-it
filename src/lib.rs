extern crate proc_macro;

use std::collections::HashMap;

use proc_macro2::{Span, TokenStream, TokenTree};

use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, spanned::Spanned, Expr, ExprArray, ExprMatch, Ident, Pat,
    PatRange, RangeLimits, Token,
};

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
enum UType {
    N,
    P,
    U,
    False,
    None,
    Literal(isize),
}

impl std::fmt::Display for UType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UType::N => write!(f, "N"),
            UType::P => write!(f, "P"),
            UType::U => write!(f, "U"),
            UType::False => write!(f, "False"),
            UType::None => write!(f, ""),
            UType::Literal(_) => write!(f, ""),
        }
    }
}

struct UNumIt {
    range: Vec<isize>,
    arms: HashMap<UType, Box<Expr>>,
    expr: Box<Expr>,
}

fn range_boundary(val: &Option<Box<Expr>>) -> syn::Result<Option<isize>> {
    if let Some(val) = val.clone() {
        let string = val.to_token_stream().to_string().replace(' ', "");
        let value = string
            .parse::<isize>()
            .map_err(|e| syn::Error::new(val.span(), format!("{e}: `{string}`").as_str()))?;

        Ok(Some(value))
    } else {
        Ok(None)
    }
}

impl Parse for UNumIt {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        // Try to parse as array first, then fallback to range
        let range: Vec<isize> = if input.peek(syn::token::Bracket) {
            // Parse array syntax: [1, 2, 8, 22]
            let array: ExprArray = input.parse()?;
            let mut vals = array
                .elems
                .iter()
                .map(|expr| {
                    let raw = expr.to_token_stream().to_string();
                    let norm = raw.replace([' ', '_'], "");
                    norm.parse::<isize>().map_err(|e| {
                        syn::Error::new(
                            expr.span(),
                            format!("invalid number in array: {e}: `{raw}` (normalized `{norm}`)"),
                        )
                    })
                })
                .collect::<syn::Result<Vec<isize>>>()?;
            vals.sort();
            vals.dedup();
            vals
        } else {
            // Parse range syntax: 1..10 or 1..=10
            let range: PatRange = input.parse()?;
            let start = range_boundary(&range.start)?.unwrap_or(0);
            let end = range_boundary(&range.end)?.unwrap_or(isize::MAX);
            match &range.limits {
                RangeLimits::HalfOpen(_) => (start..end).collect(),
                RangeLimits::Closed(_) => (start..=end).collect(),
            }
        };

        input.parse::<Token![,]>()?;
        let matcher: ExprMatch = input.parse()?;

        let mut arms = HashMap::new();

        for arm in matcher.arms.iter() {
            let u_type = match &arm.pat {
                Pat::Ident(t) => match t.ident.to_token_stream().to_string().as_str() {
                    "N" => UType::N,
                    "P" => UType::P,
                    "U" => UType::U,
                    "False" => UType::False,
                    _ => {
                        return Err(syn::Error::new(
                            t.span(),
                            "expected idents N | P | U | False | _",
                        ))
                    }
                },
                Pat::Lit(lit_expr) => {
                    // Parse literal numbers in match arms (normalize spaces & underscores; base-10 only)
                    let raw = lit_expr.to_token_stream().to_string();
                    let norm = raw.replace([' ', '_'], "");
                    if norm.starts_with("0x") || norm.starts_with("0b") || norm.starts_with("0o") {
                        return Err(syn::Error::new(
                            lit_expr.span(),
                            format!("unsupported non-decimal literal `{raw}`"),
                        ));
                    }
                    let value = norm.parse::<isize>().map_err(|e| {
                        syn::Error::new(
                            lit_expr.span(),
                            format!("invalid literal: {e}: `{raw}` (normalized `{norm}`)"),
                        )
                    })?;
                    UType::Literal(value)
                }
                Pat::Wild(_) => UType::None,
                _ => return Err(syn::Error::new(arm.pat.span(), "expected ident")),
            };
            let arm_expr = arm.body.clone();
            if arms.insert(u_type, arm_expr.clone()).is_some() {
                return Err(syn::Error::new(arm_expr.span(), "duplicate type"));
            }
        }

        if arms.get(&UType::P).and(arms.get(&UType::U)).is_some() {
            return Err(syn::Error::new(
                matcher.span(),
                "ambiguous type, don't use P and U in the same macro call",
            ));
        }

        // Check for conflict between literal 0 and False (they represent the same value in typenum)
        if arms
            .get(&UType::Literal(0))
            .and(arms.get(&UType::False))
            .is_some()
        {
            return Err(syn::Error::new(
                matcher.span(),
                "ambiguous type, don't use literal 0 and False in the same macro call (they represent the same value)",
            ));
        }

        let expr = matcher.expr;

        Ok(UNumIt { range, arms, expr })
    }
}

fn make_match_arm(i: &isize, body: &Expr, u_type: UType) -> TokenStream {
    let match_expr = quote!(#i);

    // Determine the typenum type for all cases
    let i_str = if *i != 0 {
        i.abs().to_string()
    } else {
        Default::default()
    };

    // Determine the type variant based on UType
    let u_type_for_typenum = match u_type {
        UType::Literal(0) => UType::False,
        UType::Literal(val) if val < 0 => UType::N,
        UType::Literal(val) if val > 0 => UType::P,
        _ => u_type,
    };

    let typenum_type = TokenTree::Ident(Ident::new(
        format!("{}{}", u_type_for_typenum, i_str).as_str(),
        Span::mixed_site(),
    ));
    let type_variant = quote!(typenum::consts::#typenum_type);

    // All match arms get NumType and use body as-is (no pattern replacement)
    let body_tokens = body.to_token_stream();

    quote! {
        #match_expr => {
            type NumType = #type_variant;
            #body_tokens
        },
    }
}

/// matches `typenum::consts` in a given range or array
///
/// use with an open or closed range, or an array of arbitrary numbers
///
/// use `P` | `N` | `U` | `False` | `_` or literals `1` | `-1` as match arms
///
/// a `NumType` type alias is available in each match arm,
/// resolving to the specific typenum type for that value.
/// Use `NumType` to reference the resolved type in the match arm body.
///
/// ## Example (range)
///
/// ```
/// let x = 3;
///
/// u_num_it::u_num_it!(1..10, match x {
///     U => {
///         // NumType is typenum::consts::U3 when x=3
///         let val = NumType::new();
///         println!("{:?}", val);
///         // UInt { msb: UInt { msb: UTerm, lsb: B1 }, lsb: B1 }
///
///         use typenum::ToInt;
///         let num: usize = NumType::to_int();
///         assert_eq!(num, 3);
///     }
/// })
/// ```
///
/// ## Example (array)
///
/// ```
/// let x = 8;
///
/// u_num_it::u_num_it!([1, 2, 8, 22], match x {
///     P => {
///         // NumType is typenum::consts::P8 when x=8
///         use typenum::ToInt;
///         let num: i32 = NumType::to_int();
///         assert_eq!(num, 8);
///     }
/// })
/// ```
///
/// ## Example (negative literal)
/// ```
/// let result = u_num_it::u_num_it!(-5..=5, match -3 {
///     -3 => {
///         use typenum::ToInt;
///         let n: i32 = NumType::to_int();
///         assert_eq!(n, -3);
///         "ok"
///     },
///     N => "neg",
///     _ => "other"
/// });
/// assert_eq!(result, "ok");
/// ```
#[proc_macro]
pub fn u_num_it(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let UNumIt { range, arms, expr } = parse_macro_input!(tokens as UNumIt);

    let pos_u = arms.contains_key(&UType::U);

    let expanded_arms = range.iter().filter_map(|i| {
        // First check if there's a specific literal match for this number
        if let Some(body) = arms.get(&UType::Literal(*i)) {
            return Some(make_match_arm(i, body, UType::Literal(*i)));
        }

        // Otherwise, use the general type patterns
        match i {
            0 => arms
                .get(&UType::False)
                .map(|body| make_match_arm(i, body, UType::False)),
            i if *i < 0 => arms
                .get(&UType::N)
                .map(|body| make_match_arm(i, body, UType::N)),
            i if *i > 0 => {
                if pos_u {
                    arms.get(&UType::U)
                        .map(|body| make_match_arm(i, body, UType::U))
                } else {
                    arms.get(&UType::P)
                        .map(|body| make_match_arm(i, body, UType::P))
                }
            }
            _ => unreachable!(),
        }
    });

    let fallback = arms
        .get(&UType::None)
        .map(|body| {
            quote! {
                _ => {
                    #body
                },
            }
        })
        .unwrap_or_else(|| {
            let first = range.first().unwrap_or(&0);
            let last = range.last().unwrap_or(&0);
            quote! {
                i => unreachable!("{i} not in range {}..={}", #first, #last),
            }
        });

    let expanded = quote! {
        match #expr {
            #(#expanded_arms)*
            #fallback
        }
    };

    proc_macro::TokenStream::from(expanded)
}
