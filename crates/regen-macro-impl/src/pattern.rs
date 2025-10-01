use std::ops::Bound;

use crate::{expr::eval_as_range, pattern_char::PatternChar};
use syn::{parse::Parse, punctuated::Punctuated, spanned::Spanned};

#[derive(Debug, Clone)]
pub enum Pattern<T: PatternChar> {
    Atom(PatternAtom<T>),
    Class(PatternClass),
    Seq(PatternSeq<T>),
    Join(Box<PatternJoin<T>>),
    Or(Box<PatternOr<T>>),
    Repeat(Box<PatternRepeat<T>>),
    Collect(Box<PatternCollect<T>>),
}

impl<T: PatternChar> Pattern<T> {
    pub fn new(expr: &syn::Expr) -> Result<Self, syn::Error> {
        let p = match expr {
            syn::Expr::Lit(e) => {
                use syn::Lit::*;

                match &e.lit {
                    Str(l) => Pattern::Seq(PatternSeq::from_str(l)?),
                    ByteStr(l) => Pattern::Seq(PatternSeq::from_bstr(l)?),
                    e => Pattern::Atom(PatternAtom::Primitive(T::try_from_lit(e)?)),
                }
            }
            syn::Expr::Path(e) => Pattern::Class(PatternClass {
                path: e.path.clone(),
            }),
            syn::Expr::Range(range) => {
                let start = match &range.start {
                    Some(e) => {
                        let e = expect_lit(e)?;
                        Bound::Included(T::try_from_lit(e)?)
                    }
                    None => Bound::Unbounded,
                };

                let end = match &range.end {
                    Some(e) => {
                        let e = expect_lit(e)?;
                        let e = T::try_from_lit(e)?;
                        match range.limits {
                            syn::RangeLimits::HalfOpen(_) => Bound::Excluded(e),
                            syn::RangeLimits::Closed(_) => Bound::Included(e),
                        }
                    }
                    None => Bound::Unbounded,
                };

                Pattern::Atom(PatternAtom::Range(start, end))
            }
            syn::Expr::Binary(e) => {
                let lhs = Pattern::new(&e.left)?;
                let rhs = Pattern::new(&e.right)?;
                match &e.op {
                    syn::BinOp::Add(_) => Pattern::Join(Box::new(PatternJoin { lhs, rhs })),
                    syn::BinOp::BitOr(_) => Pattern::Or(Box::new(PatternOr { lhs, rhs })),
                    _ => {
                        return Err(syn::Error::new(
                            e.span(),
                            "Unexpected operator. `+` or `|` was expected.",
                        ));
                    }
                }
            }
            syn::Expr::Array(e) => Pattern::Seq(PatternSeq::from_array(e)?),
            syn::Expr::Macro(e) => {
                let ident = e.mac.path.require_ident()?;

                if ident == "repeat" {
                    PatternRepeat::from_mac(&e.mac)
                        .map(Box::new)
                        .map(Pattern::Repeat)?
                } else if ident == "collect" {
                    PatternCollect::from_mac(&e.mac)
                        .map(Box::new)
                        .map(Pattern::Collect)?
                } else {
                    return Err(syn::Error::new(e.span(), "Unexpected pattern."));
                }
            }
            syn::Expr::Paren(e) => Pattern::new(&e.expr)?,
            syn::Expr::Repeat(e) => PatternRepeat::from_repeat(e)
                .map(Box::new)
                .map(Pattern::Repeat)?,
            _ => return Err(syn::Error::new(expr.span(), "Unexpected expression.")),
        };

        Ok(p)
    }
}

fn expect_lit(e: &syn::Expr) -> Result<&syn::Lit, syn::Error> {
    match e {
        syn::Expr::Lit(e) => Ok(&e.lit),
        e => Err(syn::Error::new(
            e.span(),
            "Unexpected expression. literal was expected.",
        )),
    }
}

impl<T: PatternChar> From<PatternAtom<T>> for Pattern<T> {
    fn from(value: PatternAtom<T>) -> Self {
        Pattern::Atom(value)
    }
}

// PatternAtom ::= char | num
#[derive(Debug, Clone)]
pub enum PatternAtom<T: PatternChar> {
    Primitive(T),
    Range(Bound<T>, Bound<T>),
}

#[derive(Debug, Clone)]
pub struct PatternClass {
    pub path: syn::Path,
}

// PatternSeq ::= array | bstr | str
#[derive(Debug, Clone)]
pub struct PatternSeq<T: PatternChar> {
    pub atoms: Vec<PatternAtom<T>>,
}

impl<T: PatternChar> PatternSeq<T> {
    fn from_str(str: &syn::LitStr) -> syn::Result<Self> {
        let atoms = str
            .value()
            .chars()
            .map(|e| T::try_from_char(e).map(PatternAtom::Primitive))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|m| syn::Error::new(str.span(), m))?;

        if atoms.is_empty() {
            Err(syn::Error::new(
                str.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { atoms })
        }
    }

    fn from_bstr(str: &syn::LitByteStr) -> syn::Result<Self> {
        let atoms: Vec<PatternAtom<T>> = str
            .value()
            .iter()
            .map(|&e| T::try_from_u8(e).map(PatternAtom::Primitive))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|m| syn::Error::new(str.span(), m))?;

        if atoms.is_empty() {
            Err(syn::Error::new(
                str.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { atoms })
        }
    }

    fn from_array(arr: &syn::ExprArray) -> syn::Result<Self> {
        let atoms = arr
            .elems
            .iter()
            .map(|e| {
                let syn::Expr::Lit(e) = e else {
                    return Err(syn::Error::new(
                        e.span(),
                        "Array pattern element must be atom literal.",
                    ));
                };

                T::try_from_lit(&e.lit).map(PatternAtom::Primitive)
            })
            .collect::<Result<Vec<_>, _>>()?;

        if atoms.is_empty() {
            Err(syn::Error::new(
                arr.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { atoms })
        }
    }
}

// PatternJoin ::= pattern + "+" +  pattern
#[derive(Debug, Clone)]
pub struct PatternJoin<T: PatternChar> {
    pub lhs: Pattern<T>,
    pub rhs: Pattern<T>,
}

// PatternOr ::= pattern + "|" + pattern
#[derive(Debug, Clone)]
pub struct PatternOr<T: PatternChar> {
    pub lhs: Pattern<T>,
    pub rhs: Pattern<T>,
}

// PatternRepeat ::=  "[" + pattern + ";" + range "]" "repeat!(" + pattern ")"  | "repeat!(" + pattern + "," + range + ")"
#[derive(Debug, Clone)]
pub struct PatternRepeat<T: PatternChar> {
    pub pattern: Pattern<T>,
    pub start: Bound<usize>,
    pub end: Bound<usize>,
}

impl<T: PatternChar> PatternRepeat<T> {
    fn from_repeat(e: &syn::ExprRepeat) -> syn::Result<Self> {
        let (start, end) = eval_as_range(&e.len)?;

        Ok(Self {
            pattern: Pattern::new(&e.expr)?,
            start,
            end,
        })
    }

    fn from_mac(mac: &syn::Macro) -> syn::Result<Self> {
        let e = mac.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated)?;

        let r = match e.len() {
            1 => {
                let pattern = Pattern::new(&e[0])?;
                PatternRepeat {
                    pattern,
                    start: Bound::Unbounded,
                    end: Bound::Unbounded,
                }
            }
            2 => {
                let pattern = Pattern::new(&e[0])?;
                let (start, end) = eval_as_range(&e[1])?;
                PatternRepeat {
                    pattern,
                    start,
                    end,
                }
            }
            _ => {
                return Err(syn::Error::new(
                    e.span(),
                    "One or two arguments were expected.",
                ));
            }
        };

        Ok(r)
    }
}

// PatternCollect ::= "collect!(" + path  + "," + pattern ")"
#[derive(Debug, Clone)]
pub struct PatternCollect<T: PatternChar> {
    pub field: String,
    pub pattern: Pattern<T>,
}

impl<T: PatternChar> PatternCollect<T> {
    fn from_mac(mac: &syn::Macro) -> syn::Result<Self> {
        let e = mac.parse_body_with(parser_fn(move |input| {
            let member = syn::Member::parse(input)?;
            let _ = <syn::Token![,]>::parse(input)?;
            let e = syn::Expr::parse(input)?;
            let pattern = Pattern::new(&e)?;

            if !input.is_empty() {
                return Err(input.error("Unexpected arguments of `collect!`."));
            }

            let field = match member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            };

            Ok(PatternCollect { field, pattern })
        }))?;

        Ok(e)
    }
}

fn parser_fn<F, T>(f: F) -> F
where
    F: for<'a> FnOnce(&'a syn::parse::ParseBuffer<'a>) -> T,
{
    f
}
