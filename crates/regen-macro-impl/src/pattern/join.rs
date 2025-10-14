use crate::{
    match_graph::{BuildContext, Builder, MatchPattern, StateId},
    pattern::{PatternKind, PatternTag, ResolvedPatternTag},
    pattern_char::PatternChar,
};

// PatternJoin ::= pattern + "+" +  pattern
#[derive(Debug, Clone)]
pub struct PatternJoin<T: PatternChar, K: PatternKind = PatternTag> {
    pub lhs: K::Pattern<T>,
    pub rhs: K::Pattern<T>,
}

impl<T: PatternChar> MatchPattern<T> for PatternJoin<T, ResolvedPatternTag> {
    fn insert(
        &self,
        builder: &mut Builder<T>,
        context: &mut BuildContext,
        from: StateId,
    ) -> StateId {
        let state = self.lhs.insert(builder, context, from);
        self.rhs.insert(builder, context, state)
    }
}
