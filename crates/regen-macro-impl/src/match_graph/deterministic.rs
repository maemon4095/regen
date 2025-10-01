use super::nondeterministic;
use std::collections::{BTreeMap, BTreeSet, HashSet};

use crate::{
    match_graph::MatchProp,
    pattern_char::PatternChar,
    util::{IntervalMap, interval_map::store::Unique},
};

#[derive(Debug)]
pub struct MatchGraph<T: PatternChar> {
    states: Vec<MatchState<T>>,
}

impl<T: PatternChar> MatchGraph<T> {
    pub fn states(&self) -> &[MatchState<T>] {
        &self.states
    }

    pub(super) fn from_nondeterministic(graph: &nondeterministic::MatchGraph<T>) -> Self {
        let initial_state = epsilon_closure(graph, [0]);
        let mut context = ConvertionContext {
            map: BTreeMap::from_iter([(initial_state.clone(), 0)]),
            unchecked: vec![initial_state],
            states: vec![MatchState::new()],
        };

        while let Some(closure) = context.unchecked.pop() {
            let id = context.get_id(&closure).unwrap();
            let branches = create_branches(&mut context, graph, &closure);
            let state = &mut context.states[id];
            state.branches = branches;
            state.assoc = closure
                .states
                .iter()
                .flat_map(|i| &graph.states[*i].assoc)
                .copied()
                .collect();
            state.assoc.sort();

            state.collects = closure
                .states
                .iter()
                .flat_map(|i| &graph.states[*i].collects)
                .cloned()
                .collect();

            state.props = closure
                .states
                .iter()
                .flat_map(|i| &graph.states[*i].props)
                .cloned()
                .collect();
        }

        Self {
            states: context.states,
        }
    }
}

fn create_branches<T: PatternChar>(
    context: &mut ConvertionContext<T>,
    graph: &nondeterministic::MatchGraph<T>,
    state: &EpsilonClosure,
) -> MatchBranches<T> {
    let mut branches: nondeterministic::MatchBranches<T> = Default::default();
    for &s in state.states.iter() {
        branches.append(&graph.states[s].branches)
    }

    let mut map = IntervalMap::new();
    for (min, max, s) in branches.iter() {
        if s.is_empty() {
            continue;
        }
        let closure = epsilon_closure(graph, s.iter().copied());
        let id = context.get_id(&closure);

        let id = match id {
            Some(v) => v,
            None => context.register_and_push(closure),
        };

        map.insert_item((min.copied(), max.copied()), &id);
    }

    MatchBranches { map }
}

fn epsilon_closure<T: PatternChar>(
    graph: &nondeterministic::MatchGraph<T>,
    state: impl IntoIterator<Item = usize>,
) -> EpsilonClosure {
    let mut reachable = BTreeSet::new();
    let mut unchecked: Vec<_> = state.into_iter().collect();

    while let Some(s) = unchecked.pop() {
        if !reachable.insert(s) {
            continue;
        }

        unchecked.extend_from_slice(&graph.states[s].epsilon_transitions);
    }

    EpsilonClosure { states: reachable }
}

#[derive(Debug)]
pub struct MatchState<T: PatternChar> {
    branches: MatchBranches<T>,
    assoc: Vec<usize>,
    collects: HashSet<MatchProp>,
    props: HashSet<MatchProp>,
}

impl<T: PatternChar> MatchState<T> {
    fn new() -> Self {
        Self {
            branches: MatchBranches::new(),
            assoc: Vec::new(),
            collects: HashSet::new(),
            props: HashSet::new(),
        }
    }

    pub fn branches(&self) -> &MatchBranches<T> {
        &self.branches
    }

    pub fn assoc(&self) -> &[usize] {
        &self.assoc
    }

    pub fn collects(&self) -> &HashSet<MatchProp> {
        &self.collects
    }

    pub fn props(&self) -> &HashSet<MatchProp> {
        &self.props
    }
}

#[derive(Debug)]
pub struct MatchBranches<T: PatternChar> {
    map: IntervalMap<T, usize, Unique>,
}

impl<T: PatternChar> MatchBranches<T> {
    fn new() -> Self {
        Self {
            map: IntervalMap::new(),
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Option<&T>, Option<&T>, &Option<usize>)> {
        self.map.iter()
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
struct EpsilonClosure {
    states: BTreeSet<usize>,
}

struct ConvertionContext<T: PatternChar> {
    states: Vec<MatchState<T>>,
    map: BTreeMap<EpsilonClosure, usize>,
    unchecked: Vec<EpsilonClosure>,
}

impl<T: PatternChar> ConvertionContext<T> {
    fn get_id(&mut self, closure: &EpsilonClosure) -> Option<usize> {
        self.map.get(closure).copied()
    }

    fn register_and_push(&mut self, closure: EpsilonClosure) -> usize {
        let id = self.states.len();
        self.map.insert(closure.clone(), id);
        self.unchecked.push(closure);
        self.states.push(MatchState::new());
        id
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let ndgraph = nondeterministic::MatchGraph::<u8>::new();
        dbg!(&ndgraph);
        let graph = MatchGraph::from_nondeterministic(&ndgraph);
        dbg!(graph);
    }
}
