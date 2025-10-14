mod atom;
mod cls;
mod collect;
mod join;
mod or;
mod repeat;
mod seq;

use crate::declares::Declares;
use crate::{match_graph::MatchPattern, pattern_char::PatternChar};
use __internal::{BelongTo, PatternKind};
use std::collections::HashMap;
use std::ops::Bound;
use syn::spanned::Spanned;

pub use atom::PatternAtom;
pub use cls::PatternClass;
pub use collect::PatternCollect;
pub use join::PatternJoin;
pub use or::PatternOr;
pub use repeat::PatternRepeat;
pub use seq::PatternSeq;

#[derive(Debug, Clone)]
pub struct PatternTag;
impl PatternKind for PatternTag {
    type Pattern<T: PatternChar> = Pattern<T>;
}

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

impl<T: PatternChar> BelongTo for Pattern<T> {
    type Kind = PatternTag;
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

#[derive(Debug, Clone)]
pub struct ResolvedPatternTag;

impl PatternKind for ResolvedPatternTag {
    type Pattern<T: PatternChar> = ResolvedPattern<T>;
}

#[derive(Debug, Clone)]
pub enum ResolvedPattern<T: PatternChar> {
    Atom(PatternAtom<T>),
    Seq(PatternSeq<T, ResolvedPatternTag>),
    Join(Box<PatternJoin<T, ResolvedPatternTag>>),
    Or(Box<PatternOr<T, ResolvedPatternTag>>),
    Repeat(Box<PatternRepeat<T, ResolvedPatternTag>>),
    Collect(Box<PatternCollect<T, ResolvedPatternTag>>),
}

impl<T: PatternChar> BelongTo for ResolvedPattern<T> {
    type Kind = ResolvedPatternTag;
}

impl<T: PatternChar> MatchPattern<T> for ResolvedPattern<T> {
    fn insert(
        &self,
        builder: &mut crate::match_graph::Builder<T>,
        context: &mut crate::match_graph::BuildContext,
        from: crate::match_graph::StateId,
    ) -> crate::match_graph::StateId {
        match self {
            ResolvedPattern::Atom(p) => p.insert(builder, context, from),
            ResolvedPattern::Seq(p) => p.insert(builder, context, from),
            ResolvedPattern::Join(p) => p.insert(builder, context, from),
            ResolvedPattern::Or(p) => p.insert(builder, context, from),
            ResolvedPattern::Repeat(p) => p.insert(builder, context, from),
            ResolvedPattern::Collect(p) => p.insert(builder, context, from),
        }
    }
}

pub struct ResolveEnv<T: PatternChar> {
    variables: HashMap<String, ResolvedPattern<T>>,
}

impl<T: PatternChar> ResolveEnv<T> {
    pub fn empty() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    pub fn new(parent: &ResolveEnv<T>, declares: &Declares<T>) -> syn::Result<Self> {
        let mut env = Self {
            variables: parent.variables.clone(),
        };

        for (name, pattern) in declares.variables() {
            let p = env.resolve(pattern)?;
            env.variables.insert(name.clone(), p);
        }

        Ok(env)
    }

    pub fn variable(&self, name: &str) -> Option<&ResolvedPattern<T>> {
        self.variables.get(name)
    }

    pub fn resolve(&self, pattern: &Pattern<T>) -> syn::Result<ResolvedPattern<T>> {
        let p = match pattern {
            Pattern::Atom(p) => ResolvedPattern::Atom(p.clone()),
            Pattern::Seq(p) => {
                let patterns = p
                    .patterns
                    .iter()
                    .map(|e| self.resolve(e))
                    .collect::<Result<Vec<_>, _>>()?;

                ResolvedPattern::Seq(PatternSeq { patterns })
            }
            Pattern::Join(p) => {
                let lhs = self.resolve(&p.lhs)?;
                let rhs = self.resolve(&p.rhs)?;
                let p = PatternJoin { lhs, rhs };
                ResolvedPattern::Join(Box::new(p))
            }
            Pattern::Or(p) => {
                let lhs = self.resolve(&p.lhs)?;
                let rhs = self.resolve(&p.rhs)?;
                let p = PatternOr { lhs, rhs };
                ResolvedPattern::Or(Box::new(p))
            }
            Pattern::Repeat(p) => {
                let pattern = self.resolve(&p.pattern)?;
                let p = PatternRepeat {
                    pattern,
                    start: p.start,
                    end: p.end,
                };
                ResolvedPattern::Repeat(Box::new(p))
            }
            Pattern::Collect(p) => {
                let pattern = self.resolve(&p.pattern)?;
                let p = PatternCollect {
                    pattern,
                    field: p.field.clone(),
                };
                ResolvedPattern::Collect(Box::new(p))
            }
            Pattern::Class(c) => {
                let name = c.path.require_ident()?.to_string();
                let Some(p) = self.variable(&name) else {
                    return Err(syn::Error::new(c.path.span(), "Undeclared variable."));
                };
                p.clone()
            }
        };

        Ok(p)
    }
}

mod __internal {
    use crate::pattern_char::PatternChar;

    pub trait PatternKind {
        type Pattern<T: PatternChar>: BelongTo<Kind = Self>;
    }

    pub trait BelongTo {
        type Kind: PatternKind;
    }
}
