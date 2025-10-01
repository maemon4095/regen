use std::ops::Bound;
use syn::spanned::Spanned;

use crate::util::range::is_range_empty;

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
