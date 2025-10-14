use crate::{
    linkedlist::LinkedList,
    match_graph::{MatchProp, deterministic, nondeterministic},
    pattern_char::PatternChar,
    util::interval_map::Interval,
};

#[derive(Debug, Clone, Copy)]
pub struct StateId(usize);

pub struct Builder<T: PatternChar> {
    states: Vec<nondeterministic::MatchState<T>>,
}

impl<T: PatternChar> Builder<T> {
    pub fn new() -> Self {
        Self {
            states: vec![Default::default()],
        }
    }

    pub fn initial_state(&self) -> StateId {
        StateId(0)
    }

    pub fn alloc_state(&mut self, context: &mut BuildContext) -> StateId {
        let next = self.states.len();

        let state = nondeterministic::MatchState {
            branches: Default::default(),
            epsilon_transitions: Default::default(),
            assoc: Default::default(),
            collects: context.collects.to_vec(),
            props: context.props.clone(),
        };

        self.states.push(state);
        StateId(next)
    }

    pub fn insert_epsilon_transition(&mut self, from: StateId, to: StateId) {
        self.states[from.0].epsilon_transitions.push(to.0);
    }

    pub fn add(&mut self, assoc: usize, pattern: &impl MatchPattern<T>) {
        let initial = self.initial_state();
        let mut context = BuildContext {
            assoc,
            collects: LinkedList::empty(),
            props: &mut Vec::new(),
        };

        let state = pattern.insert(self, &mut context, initial);
        self.states[state.0].assoc.push(assoc);
    }

    pub fn insert_atom(
        &mut self,
        context: &mut BuildContext,
        from: StateId,
        range: impl Interval<T>,
    ) -> StateId {
        let state = self.alloc_state(context);
        self.states[from.0].branches.insert(range, &[state.0]);
        state
    }

    pub fn insert_repeat(
        &mut self,
        context: &mut BuildContext,
        from: StateId,
        pattern: &impl MatchPattern<T>,
    ) -> StateId {
        let state = self.alloc_state(context);
        self.insert_epsilon_transition(from, state);
        let end = pattern.insert(self, context, state);
        self.insert_epsilon_transition(end, state);
        state
    }

    pub fn insert_or(
        &mut self,
        context: &mut BuildContext,
        from: StateId,
        pattern0: &impl MatchPattern<T>,
        pattern1: &impl MatchPattern<T>,
    ) -> StateId {
        let state0 = pattern0.insert(self, context, from);
        let state1 = pattern1.insert(self, context, from);
        let state = self.alloc_state(context);

        self.insert_epsilon_transition(state0, state);
        self.insert_epsilon_transition(state1, state);

        state
    }

    pub fn insert_collect(
        &mut self,
        context: &mut BuildContext,
        from: StateId,
        field: &str,
        pattern: &impl MatchPattern<T>,
    ) -> StateId {
        let prop = MatchProp {
            assoc: context.assoc,
            field: field.to_string(),
        };
        let collects = context.collects.append(prop.clone());
        context.props.push(prop);

        let mut ctx = BuildContext {
            assoc: context.assoc,
            collects,
            props: context.props,
        };

        pattern.insert(self, &mut ctx, from)
    }

    pub fn build(self) -> deterministic::MatchGraph<T> {
        let ndgraph = nondeterministic::MatchGraph {
            states: self.states,
        };

        deterministic::MatchGraph::from_nondeterministic(&ndgraph)
    }
}

pub trait MatchPattern<T: PatternChar> {
    fn insert(
        &self,
        builder: &mut Builder<T>,
        context: &mut BuildContext,
        from: StateId,
    ) -> StateId;
}

pub struct BuildContext<'a> {
    assoc: usize,
    collects: LinkedList<MatchProp>,
    props: &'a mut Vec<MatchProp>,
}
