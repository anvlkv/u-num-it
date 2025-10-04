extern crate proc_macro;

use std::{collections::HashMap, str::FromStr};

use proc_macro2::{Group, Literal, Span, TokenStream, TokenTree};

use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, spanned::Spanned, Expr, ExprMatch, Ident, Pat, PatRange,
    RangeLimits, Token,
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
        let range: PatRange = input.parse()?;

        let start = range_boundary(&range.start)?.unwrap_or(0);
        let end = range_boundary(&range.end)?.unwrap_or(isize::MAX);

        let range = match &range.limits {
            RangeLimits::HalfOpen(_) => (start..end).collect(),
            RangeLimits::Closed(_) => (start..=end).collect(),
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
                            "exepected idents N | P | U, False or _",
                        ))
                    }
                },
                Pat::Lit(lit_expr) => {
                    // Parse literal numbers in match arms
                    let lit_str = lit_expr.to_token_stream().to_string();
                    let value = lit_str.parse::<isize>().map_err(|e| {
                        syn::Error::new(lit_expr.span(), format!("invalid literal: {e}"))
                    })?;
                    UType::Literal(value)
                }
                Pat::Wild(_) => UType::None,
                _ => return Err(syn::Error::new(arm.pat.span(), "exepected ident")),
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
        if arms.get(&UType::Literal(0)).and(arms.get(&UType::False)).is_some() {
            return Err(syn::Error::new(
                matcher.span(),
                "ambiguous type, don't use literal 0 and False in the same macro call (they represent the same value)",
            ));
        }

        let expr = matcher.expr;

        Ok(UNumIt { range, arms, expr })
    }
}

fn make_body_variant(body: TokenStream, type_variant: TokenStream, u_type: UType) -> TokenStream {
    let tokens = body.into_iter().fold(vec![], |mut acc, token| {
        let type_variant = type_variant.clone();
        match token {
            TokenTree::Ident(ref ident) => {
                if *ident == u_type.to_string() {
                    acc.extend(quote!(#type_variant).to_token_stream());
                } else {
                    acc.push(token);
                }
            }
            TokenTree::Group(ref group) => {
                let inner = make_body_variant(group.stream(), type_variant, u_type);
                acc.push(TokenTree::Group(Group::new(group.delimiter(), inner)));
            }
            _ => acc.push(token),
        };
        acc
    });

    quote! {#(#tokens)*}
}

fn make_match_arm(i: &isize, body: &Expr, u_type: UType) -> TokenStream {
    let match_expr = TokenTree::Literal(Literal::from_str(i.to_string().as_str()).unwrap());
    
    // For literal types, use the body as-is without type replacement
    if let UType::Literal(_) = u_type {
        let body_tokens = body.to_token_stream();
        return quote! {
            #match_expr => {
                #body_tokens
            },
        };
    }
    
    // For type patterns (N, P, U, False), perform type replacement
    let i_str = if *i != 0 {
        i.abs().to_string()
    } else {
        Default::default()
    };
    let typenum_type = TokenTree::Ident(Ident::new(
        format!("{}{}", u_type, i_str).as_str(),
        Span::mixed_site(),
    ));
    let type_variant = quote!(typenum::consts::#typenum_type);
    let body_variant = make_body_variant(body.to_token_stream(), type_variant, u_type);

    quote! {
        #match_expr => {
            #body_variant
        },
    }
}

/// matches `typenum::consts` in a given range
///
/// use with an open or closed range
///
/// use `P` | `N` | `U` | `False` | `_` as match arms
///
/// ## Example
///
/// ```
/// let x = 3;
///
/// u_num_it::u_num_it!(1..10, match x {
///     U => {
///         let val = U::new();
///         println!("{:?}", val);
///         // UInt { msb: UInt { msb: UTerm, lsb: B1 }, lsb: B1 }
///     }
/// })
/// ```
#[proc_macro]
pub fn u_num_it(tokens: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let UNumIt { range, arms, expr } = parse_macro_input!(tokens as UNumIt);

    let pos_u = arms.get(&UType::U).is_some();

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
                }
            }
        })
        .unwrap_or_else(|| {
            let first = range.first().unwrap_or(&0);
            let last = range.last().unwrap_or(&0);
            quote! {
                i => unreachable!("{i} is not in range {}-{:?}", #first, #last)
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
