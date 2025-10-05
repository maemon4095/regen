use std::{collections::BTreeSet, ops::Bound};

use crate::{
    linkedlist::LinkedList,
    match_graph::MatchProp,
    pattern::{
        Pattern, PatternAtom, PatternCollect, PatternJoin, PatternOr, PatternRepeat, PatternSeq,
    },
    pattern_char::PatternChar,
    resolved_pattern::ResolvedPattern,
    util::{IntervalMap, interval_map::Interval, range::is_range_empty},
};

// 関数を使ったclassも、含む含まないの全パターン列挙することでDFAにできるが、ステートが爆発的に増えるため追加しない。

#[derive(Debug)]
pub struct MatchGraph<T: PatternChar> {
    pub(super) states: Vec<MatchState<T>>,
}

impl<T: PatternChar> MatchGraph<T> {
    pub fn new() -> Self {
        Self {
            states: vec![Default::default()],
        }
    }

    pub fn add(&mut self, assoc: usize, pattern: ResolvedPattern<T>) {
        let s = self.insert(0, assoc, &pattern, LinkedList::empty(), &mut Vec::new());
        self.states[s].assoc.push(assoc);
    }

    fn alloc_state(&mut self, collects: &LinkedList<MatchProp>, props: &Vec<MatchProp>) -> usize {
        let next = self.states.len();

        let state = MatchState {
            branches: Default::default(),
            epsilon_transitions: Default::default(),
            assoc: Default::default(),
            collects: collects.to_vec(),
            props: props.clone(),
        };

        self.states.push(state);
        next
    }

    pub fn insert(
        &mut self,
        state: usize,
        assoc: usize,
        pattern: &ResolvedPattern<T>,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        self.insert_impl(state, assoc, pattern.pattern(), collects, props)
    }

    fn insert_impl(
        &mut self,
        state: usize,
        assoc: usize,
        pattern: &Pattern<T>,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        match pattern {
            Pattern::Atom(p) => self.insert_atom(state, p, collects, props),
            Pattern::Seq(p) => self.insert_seq(state, assoc, p, collects, props),
            Pattern::Join(p) => self.insert_join(state, assoc, p, collects, props),
            Pattern::Or(p) => self.insert_or(state, assoc, p, collects, props),
            Pattern::Repeat(p) => self.insert_repeat(state, assoc, p, collects, props),
            Pattern::Collect(p) => self.insert_collect(state, assoc, p, collects, props),
            Pattern::Class(_) => unreachable!(),
        }
    }

    fn insert_atom(
        &mut self,
        state: usize,
        pattern: &PatternAtom<T>,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        let dst = self.alloc_state(&collects, props);
        let range = match pattern {
            PatternAtom::Primitive(p) => (Bound::Included(p), Bound::Included(p)),
            PatternAtom::Range(s, e) => (s.as_ref(), e.as_ref()),
        };
        let range = crate::util::range::to_ropen(range);
        self.states[state].branches.insert(range, &[dst]);
        dst
    }

    fn insert_seq(
        &mut self,
        mut state: usize,
        assoc: usize,
        pattern: &PatternSeq<T>,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        for p in &pattern.patterns {
            state = self.insert_impl(state, assoc, p, collects.clone(), props);
        }
        state
    }

    fn insert_join(
        &mut self,
        state: usize,
        assoc: usize,
        pattern: &PatternJoin<T>,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        let state = self.insert_impl(state, assoc, &pattern.lhs, collects.clone(), props);

        self.insert_impl(state, assoc, &pattern.rhs, collects, props)
    }

    fn insert_or(
        &mut self,
        state: usize,
        assoc: usize,
        pattern: &PatternOr<T>,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        let state0 = self.insert_impl(state, assoc, &pattern.lhs, collects.clone(), props);
        let state1 = self.insert_impl(state, assoc, &pattern.rhs, collects.clone(), props);
        let state = self.alloc_state(&collects, props);

        self.states[state0].epsilon_transitions.push(state);
        self.states[state1].epsilon_transitions.push(state);

        state
    }

    fn insert_repeat_n(
        &mut self,
        mut state: usize,
        assoc: usize,
        pattern: &Pattern<T>,
        count: usize,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        for _ in 0..count {
            state = self.insert_impl(state, assoc, pattern, collects.clone(), props);
        }
        state
    }

    fn insert_repeat(
        &mut self,
        state: usize,
        assoc: usize,
        pattern: &PatternRepeat<T>,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        if is_range_empty(pattern.start, pattern.end) {
            panic!("Range must not be empty.")
        }

        let state = match pattern.start {
            Bound::Included(n) if n > 0 => {
                self.insert_repeat_n(state, assoc, &pattern.pattern, n, collects.clone(), props)
            }
            Bound::Excluded(n) => self.insert_repeat_n(
                state,
                assoc,
                &pattern.pattern,
                n + 1,
                collects.clone(),
                props,
            ),
            _ => {
                let s = self.alloc_state(&collects, props);
                self.states[state].epsilon_transitions.push(s);
                s
            }
        };

        match &pattern.end {
            Bound::Included(n) => {
                let mut state = state;
                let end_state = self.alloc_state(&collects, props);
                for _ in 0..(*n) {
                    state =
                        self.insert_impl(state, assoc, &pattern.pattern, collects.clone(), props);
                    self.states[state].epsilon_transitions.push(end_state);
                }
                end_state
            }
            Bound::Excluded(n) => {
                let mut state = state;
                let end_state = self.alloc_state(&collects, props);
                for _ in 1..(*n) {
                    state =
                        self.insert_impl(state, assoc, &pattern.pattern, collects.clone(), props);
                    self.states[state].epsilon_transitions.push(end_state);
                }
                end_state
            }
            Bound::Unbounded => {
                let s = self.insert_impl(state, assoc, &pattern.pattern, collects, props);
                self.states[s].epsilon_transitions.push(state);
                state
            }
        }
    }

    fn insert_collect(
        &mut self,
        state: usize,
        assoc: usize,
        pattern: &PatternCollect<T>,
        collects: LinkedList<MatchProp>,
        props: &mut Vec<MatchProp>,
    ) -> usize {
        let prop = MatchProp {
            assoc,
            field: pattern.field.clone(),
        };

        props.push(prop.clone());
        self.insert_impl(state, assoc, &pattern.pattern, collects.append(prop), props)
    }
}

#[derive(Debug)]
pub(super) struct MatchState<T: PatternChar> {
    pub(super) branches: MatchBranches<T>,
    pub(super) epsilon_transitions: Vec<usize>,
    pub(super) assoc: Vec<usize>,
    pub(super) collects: Vec<MatchProp>,
    pub(super) props: Vec<MatchProp>,
}

impl<T: PatternChar> Default for MatchState<T> {
    fn default() -> Self {
        Self {
            branches: Default::default(),
            epsilon_transitions: Default::default(),
            assoc: Default::default(),
            collects: Default::default(),
            props: Default::default(),
        }
    }
}
// 右半開区間を文字とみなして遷移テーブルを作る。各区間の最大をdivsに入れる。
#[derive(Debug)]
pub(super) struct MatchBranches<T: PatternChar> {
    map: IntervalMap<T, usize, crate::util::interval_map::store::Set>,
}

impl<T: PatternChar> MatchBranches<T> {
    pub fn insert<R: Interval<T>, I>(&mut self, interval: R, states: &I)
    where
        for<'a> &'a I: IntoIterator<Item = &'a usize>,
    {
        self.map.insert(interval, states);
    }

    pub fn iter(&self) -> impl Iterator<Item = (Option<&T>, Option<&T>, &BTreeSet<usize>)> {
        self.map.iter()
    }

    pub fn append(&mut self, other: &Self) {
        self.map.append(&other.map);
    }
}

impl<T: PatternChar> Default for MatchBranches<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}
