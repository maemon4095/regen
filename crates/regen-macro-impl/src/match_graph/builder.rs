use crate::{
    match_graph::{deterministic, nondeterministic},
    pattern_char::PatternChar,
    resolved_pattern::ResolvedPattern,
};

pub struct Builder<T: PatternChar> {
    graph: nondeterministic::MatchGraph<T>,
}

impl<T: PatternChar> Builder<T> {
    pub fn new() -> Self {
        Self {
            graph: nondeterministic::MatchGraph::new(),
        }
    }

    pub fn add(&mut self, assoc: usize, pattern: ResolvedPattern<T>) {
        self.graph.add(assoc, pattern);
    }

    pub fn build(&self) -> deterministic::MatchGraph<T> {
        deterministic::MatchGraph::from_nondeterministic(&self.graph)
    }
}
