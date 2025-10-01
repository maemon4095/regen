use std::ops::Bound;

use quote::ToTokens;
use syn::spanned::Spanned;

use crate::util::Discrete;

pub fn eval_as_usize(expr: &syn::Expr) -> syn::Result<usize> {
    match expr {
        syn::Expr::Lit(e) => match &e.lit {
            syn::Lit::Int(e) => e.base10_parse(),
            _ => Err(syn::Error::new(
                expr.span(),
                "Integer literal was expected.",
            )),
        },

        syn::Expr::Binary(e) => {
            let lhs = eval_as_usize(&e.left)?;
            let rhs = eval_as_usize(&e.right)?;
            let v = match &e.op {
                syn::BinOp::Add(_) => lhs + rhs,
                syn::BinOp::Sub(_) => lhs - rhs,
                syn::BinOp::Mul(_) => lhs * rhs,
                syn::BinOp::Div(_) => lhs / rhs,
                syn::BinOp::Rem(_) => lhs % rhs,
                syn::BinOp::BitXor(_) => lhs ^ rhs,
                syn::BinOp::BitAnd(_) => lhs & rhs,
                syn::BinOp::BitOr(_) => lhs | rhs,
                syn::BinOp::Shl(_) => lhs << rhs,
                syn::BinOp::Shr(_) => lhs >> rhs,
                _ => return Err(syn::Error::new(expr.span(), "Unsupported operator.")),
            };

            Ok(v)
        }
        syn::Expr::Paren(e) => eval_as_usize(&e.expr),
        _ => Err(syn::Error::new(expr.span(), "Unsupported expression.")),
    }
}

pub fn eval_as_range(expr: &syn::Expr) -> syn::Result<(Bound<usize>, Bound<usize>)> {
    match expr {
        syn::Expr::Range(e) => {
            let start = match &e.start {
                Some(v) => Bound::Included(eval_as_usize(v)?),
                None => Bound::Unbounded,
            };

            let end = match &e.end {
                Some(v) => match e.limits {
                    syn::RangeLimits::HalfOpen(_) => Bound::Excluded(eval_as_usize(&v)?),
                    syn::RangeLimits::Closed(_) => Bound::Included(eval_as_usize(&v)?),
                },
                None => Bound::Unbounded,
            };

            if is_range_empty(start, end) {
                return Err(syn::Error::new(expr.span(), "Range must not be empty."));
            }

            Ok((start, end))
        }
        _ => Err(syn::Error::new(
            expr.span(),
            "Range primitive was expected.",
        )),
    }
}

pub fn is_range_empty(start: Bound<usize>, end: Bound<usize>) -> bool {
    match (start, end) {
        (Bound::Included(min), Bound::Included(max)) => min > max,
        (Bound::Included(min), Bound::Excluded(max)) => min >= max,
        (Bound::Included(_), Bound::Unbounded) => false,
        (Bound::Excluded(min), Bound::Included(max)) => min >= max,
        (Bound::Excluded(min), Bound::Excluded(max)) => {
            max.checked_sub(min).map(|d| d > 1).unwrap_or(false)
        }
        (Bound::Excluded(min), Bound::Unbounded) => min == usize::MAX,
        (Bound::Unbounded, Bound::Included(_)) => false,
        (Bound::Unbounded, Bound::Excluded(max)) => max == 0,
        (Bound::Unbounded, Bound::Unbounded) => false,
    }
}

pub trait PatternChar: Ord + Copy + Eq + ToTokens + Discrete {
    fn try_from_lit(lit: &syn::Lit) -> Result<Self, syn::Error>;
    fn try_from_char(c: char) -> Result<Self, &'static str>;
    fn try_from_u8(b: u8) -> Result<Self, &'static str>;
}

impl PatternChar for char {
    fn try_from_lit(lit: &syn::Lit) -> Result<Self, syn::Error> {
        match lit {
            syn::Lit::Char(c) => Ok(c.value()),
            _ => Err(syn::Error::new(lit.span(), "char literal was expected.")),
        }
    }

    fn try_from_char(c: char) -> Result<Self, &'static str> {
        Ok(c)
    }

    fn try_from_u8(_: u8) -> Result<Self, &'static str> {
        Err("char literal was expected.")
    }
}

macro_rules! impl_pattern_primitive {
    (@int $($ty: ty),*) => {
        $(
            impl PatternChar for $ty {
                fn try_from_lit(lit: &syn::Lit) -> Result<Self, syn::Error> {
                    let v = match lit {
                        syn::Lit::Byte(c) => c.value().into(),
                        syn::Lit::Int(c) => c.base10_parse()?,
                        _ =>return Err(syn::Error::new(lit.span(), concat!(stringify!($ty), " literal was expected."))),
                    };
                    Ok(v)
                }

                fn try_from_char(_: char) -> Result<Self, &'static str> {
                    Err(concat!(stringify!($ty), " literal was expected."))
                }

                fn try_from_u8(b: u8) -> Result<Self, &'static str> {
                    Ok(b.into())
                }
            }
        )*
    };
}

impl_pattern_primitive!(@int usize, u8, u16, u32, u64);
