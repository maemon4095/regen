use syn::spanned::Spanned;

use crate::{
    Declares,
    pattern::{Pattern, PatternCollect, PatternJoin, PatternOr, PatternRepeat},
    pattern_char::PatternChar,
};

pub struct ResolvedPattern<T: PatternChar> {
    pattern: Pattern<T>,
}

impl<T: PatternChar> ResolvedPattern<T> {
    pub fn resolve(env: &Declares<T>, pattern: Pattern<T>) -> syn::Result<Self> {
        let pattern = resolve_impl(env, pattern)?;

        Ok(ResolvedPattern { pattern })
    }

    pub fn pattern(&self) -> &Pattern<T> {
        &self.pattern
    }
}

fn resolve_impl<T: PatternChar>(env: &Declares<T>, pattern: Pattern<T>) -> syn::Result<Pattern<T>> {
    let p = match pattern {
        p @ Pattern::Atom(_) => p,
        p @ Pattern::Seq(_) => p,
        Pattern::Join(p) => {
            let lhs = resolve_impl(env, p.lhs)?;
            let rhs = resolve_impl(env, p.rhs)?;
            let p = PatternJoin { lhs, rhs };
            Pattern::Join(Box::new(p))
        }
        Pattern::Or(p) => {
            let lhs = resolve_impl(env, p.lhs)?;
            let rhs = resolve_impl(env, p.rhs)?;
            let p = PatternOr { lhs, rhs };
            Pattern::Or(Box::new(p))
        }
        Pattern::Repeat(p) => {
            let pattern = resolve_impl(env, p.pattern)?;
            let p = PatternRepeat {
                pattern,
                start: p.start,
                end: p.end,
            };
            Pattern::Repeat(Box::new(p))
        }
        Pattern::Collect(p) => {
            let pattern = resolve_impl(env, p.pattern)?;
            let p = PatternCollect {
                pattern,
                field: p.field,
            };
            Pattern::Collect(Box::new(p))
        }
        Pattern::Class(c) => {
            let name = c.path.require_ident()?.to_string();
            let Some(p) = env.variables.get(&name) else {
                return Err(syn::Error::new(c.path.span(), "Undeclared variable."));
            };
            p.clone()
        }
    };

    Ok(p)
}
