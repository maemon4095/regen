use crate::{
    match_graph::{BuildContext, Builder, MatchPattern, StateId},
    pattern::{PatternKind, PatternTag, ResolvedPatternTag},
    pattern_char::PatternChar,
};

// PatternOr ::= pattern + "|" + pattern
#[derive(Debug, Clone)]
pub struct PatternOr<T: PatternChar, K: PatternKind = PatternTag> {
    pub lhs: K::Pattern<T>,
    pub rhs: K::Pattern<T>,
}

impl<T: PatternChar> MatchPattern<T> for PatternOr<T, ResolvedPatternTag> {
    fn insert(
        &self,
        builder: &mut Builder<T>,
        context: &mut BuildContext,
        from: StateId,
    ) -> StateId {
        builder.insert_or(context, from, &self.lhs, &self.rhs)
    }
}
