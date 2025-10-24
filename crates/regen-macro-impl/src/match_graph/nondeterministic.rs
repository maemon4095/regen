use crate::{
    match_graph::MatchProp,
    pattern_char::PatternChar,
    util::{IntervalMap, interval_map::Interval},
};
use std::collections::BTreeSet;

// 関数を使ったclassも、含む含まないの全パターン列挙することでDFAにできるが、ステートが爆発的に増えるため追加しない。

#[derive(Debug)]
pub struct MatchGraph<T: PatternChar> {
    pub(super) states: Vec<MatchState<T>>,
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

    pub fn combine(&mut self, other: &Self) {
        self.map.combine(&other.map);
    }
}

impl<T: PatternChar> Default for MatchBranches<T> {
    fn default() -> Self {
        Self {
            map: Default::default(),
        }
    }
}
