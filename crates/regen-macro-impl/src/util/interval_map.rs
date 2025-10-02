pub mod store;

use std::{
    collections::BTreeMap,
    ops::{Bound, Range, RangeFrom, RangeFull, RangeTo, RangeToInclusive},
};

use crate::util::{Discrete, Iterable, interval_map::store::Store};

#[derive(Debug)]
pub struct IntervalMap<K: Ord + Clone, V: Clone, C: Store<V>> {
    lower: C::Type,
    divs: BTreeMap<K, C::Type>,
}

impl<K: Ord + Clone, V: Clone, C: Store<V>> IntervalMap<K, V, C> {
    pub fn new() -> Self {
        Self {
            lower: C::create(),
            divs: Default::default(),
        }
    }

    pub fn insert_item<R: Interval<K>>(&mut self, interval: R, item: &V) {
        self.insert(interval, std::slice::from_ref(item));
    }

    pub fn insert<R: Interval<K>, I: ?Sized + Iterable<Item = V>>(
        &mut self,
        interval: R,
        items: &I,
    ) {
        let (from, to) = interval.into_ropen();
        self.insert_ropen(from, to, items);
    }

    fn insert_ropen<I: ?Sized + Iterable<Item = V>>(
        &mut self,
        from: Option<K>,
        to: Option<K>,
        items: &I,
    ) {
        let lower_bound = match &from {
            Some(k) => {
                if !self.divs.contains_key(k) {
                    let vs = self
                        .divs
                        .range(..k)
                        .last()
                        .map(|e| e.1)
                        .unwrap_or(&self.lower);

                    let mut v: C::Type = C::create();
                    C::extend(&mut v, vs.iter().cloned());

                    self.divs.insert(k.clone(), v);
                }
                Bound::Included(k)
            }
            None => Bound::Unbounded,
        };

        let upper_bound = match &to {
            Some(k) => {
                if !self.divs.contains_key(k) {
                    let vs = self
                        .divs
                        .range(..k)
                        .last()
                        .map(|e| e.1)
                        .unwrap_or(&self.lower);

                    let mut v: C::Type = C::create();
                    C::extend(&mut v, vs.iter().cloned());

                    self.divs.insert(k.clone(), v);
                }
                Bound::Excluded(k)
            }
            None => Bound::Unbounded,
        };

        if from.is_none() {
            C::extend(&mut self.lower, items.iter().cloned());
        }

        for (_, interval) in self.divs.range_mut((lower_bound, upper_bound)) {
            C::extend(interval, items.iter().cloned());
        }
    }

    pub fn append(&mut self, other: &IntervalMap<K, V, C>) {
        for (from, to, s) in other.iter() {
            self.insert_ropen(from.cloned(), to.cloned(), s);
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (Option<&K>, Option<&K>, &C::Type)> {
        self.divs
            .iter()
            .map(Some)
            .chain([None])
            .scan(Some((None, &self.lower)), |state, upper| {
                let (last_lower_bound, last_assoc) = state.take()?;
                match upper {
                    Some((upper_bound, assoc)) => {
                        *state = Some((Some(upper_bound), assoc));
                        Some((last_lower_bound, Some(upper_bound), last_assoc))
                    }
                    None => Some((last_lower_bound, None, last_assoc)),
                }
            })
    }
}

impl<K: Ord + Clone, V: Clone, C: Store<V>> Default for IntervalMap<K, V, C> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Interval<T> {
    fn into_ropen(self) -> (Option<T>, Option<T>);
}

impl<T> Interval<T> for RangeFrom<T> {
    fn into_ropen(self) -> (Option<T>, Option<T>) {
        (Some(self.start), None)
    }
}

impl<T> Interval<T> for RangeTo<T> {
    fn into_ropen(self) -> (Option<T>, Option<T>) {
        (None, Some(self.end))
    }
}

impl<T: Discrete> Interval<T> for RangeToInclusive<T> {
    fn into_ropen(self) -> (Option<T>, Option<T>) {
        (None, self.end.next_up())
    }
}

impl<T> Interval<T> for RangeFull {
    fn into_ropen(self) -> (Option<T>, Option<T>) {
        (None, None)
    }
}

impl<T> Interval<T> for Range<T> {
    fn into_ropen(self) -> (Option<T>, Option<T>) {
        (Some(self.start), Some(self.end))
    }
}
impl<T> Interval<T> for (Option<T>, Option<T>) {
    fn into_ropen(self) -> (Option<T>, Option<T>) {
        self
    }
}

#[cfg(test)]
mod test {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn test() {
        let mut map = IntervalMap::<u8, usize, store::Set>::new();

        map.insert(98..99, &[2]);

        assert!(map.iter().eq([
            (None, Some(&98), &BTreeSet::from_iter([])),
            (Some(&98), Some(&99), &BTreeSet::from_iter([2])),
            (Some(&99), None, &BTreeSet::from_iter([])),
        ]));

        map.insert(97..98, &[1]);

        assert!(map.iter().eq([
            (None, Some(&97), &BTreeSet::from_iter([])),
            (Some(&97), Some(&98), &BTreeSet::from_iter([1])),
            (Some(&98), Some(&99), &BTreeSet::from_iter([2])),
            (Some(&99), None, &BTreeSet::from_iter([])),
        ]));
    }
}
