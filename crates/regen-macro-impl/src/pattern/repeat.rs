use crate::{
    eval::eval_as_range,
    match_graph::{BuildContext, Builder, MatchPattern, StateId},
    pattern::{Pattern, PatternKind, PatternTag, ResolvedPatternTag},
    pattern_char::PatternChar,
    util::range::is_range_empty,
};
use std::ops::Bound;
use syn::{punctuated::Punctuated, spanned::Spanned as _};

// PatternRepeat ::=  "[" + pattern + ";" + range "]" "repeat!(" + pattern ")"  | "repeat!(" + pattern + "," + range + ")"
#[derive(Debug, Clone)]
pub struct PatternRepeat<T: PatternChar, K: PatternKind = PatternTag> {
    pub pattern: K::Pattern<T>,
    pub start: Bound<usize>,
    pub end: Bound<usize>,
}

impl<T: PatternChar> PatternRepeat<T> {
    pub fn from_repeat(e: &syn::ExprRepeat) -> syn::Result<Self> {
        let (start, end) = eval_as_range(&e.len)?;

        Ok(Self {
            pattern: Pattern::new(&e.expr)?,
            start,
            end,
        })
    }

    pub fn from_mac(mac: &syn::Macro) -> syn::Result<Self> {
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

impl<T: PatternChar> MatchPattern<T> for PatternRepeat<T, ResolvedPatternTag> {
    fn insert(
        &self,
        builder: &mut Builder<T>,
        context: &mut BuildContext,
        from: StateId,
    ) -> StateId {
        if is_range_empty(self.start, self.end) {
            panic!("Range must not be empty.")
        }

        let state = match self.start {
            Bound::Included(n) if n > 0 => {
                insert_repeat_n(builder, context, from, &self.pattern, n)
            }
            Bound::Excluded(n) => insert_repeat_n(builder, context, from, &self.pattern, n + 1),
            _ => from,
        };

        match &self.end {
            Bound::Included(n) => {
                let mut state = state;
                let end_state = builder.alloc_state(context);
                for _ in 0..(*n) {
                    state = self.pattern.insert(builder, context, state);
                    builder.insert_epsilon_transition(state, end_state);
                }
                end_state
            }
            Bound::Excluded(n) => {
                let mut state = state;
                let end_state = builder.alloc_state(context);
                for _ in 1..(*n) {
                    state = self.pattern.insert(builder, context, state);
                    builder.insert_epsilon_transition(state, end_state);
                }
                end_state
            }
            Bound::Unbounded => builder.insert_repeat(context, state, &self.pattern),
        }
    }
}

fn insert_repeat_n<T: PatternChar>(
    builder: &mut Builder<T>,
    context: &mut BuildContext,
    from: StateId,
    pattern: &impl MatchPattern<T>,
    count: usize,
) -> StateId {
    let mut state = from;
    for _ in 0..count {
        state = pattern.insert(builder, context, state);
    }
    state
}
