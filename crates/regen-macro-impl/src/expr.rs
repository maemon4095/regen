use syn::spanned::Spanned;

pub fn eval_as_usize(expr: &syn::Expr) -> syn::Result<usize> {
    match expr {
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
