use crate::{
    match_graph::{BuildContext, Builder, MatchPattern, StateId},
    pattern::Pattern,
    pattern_char::PatternChar,
};
use std::ops::Bound;

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

impl<T: PatternChar> MatchPattern<T> for PatternAtom<T> {
    fn insert(
        &self,
        builder: &mut Builder<T>,
        context: &mut BuildContext,
        from: StateId,
    ) -> StateId {
        let range = match self {
            Self::Primitive(p) => (Bound::Included(p), Bound::Included(p)),
            Self::Range(s, e) => (s.as_ref(), e.as_ref()),
        };
        let range = crate::util::range::to_ropen(range);
        builder.insert_atom(context, from, range)
    }
}
