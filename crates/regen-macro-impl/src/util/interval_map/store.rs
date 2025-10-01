use std::collections::BTreeSet;

use crate::util::Iterable;

pub trait Store<T> {
    type Type: Iterable<Item = T>;

    fn create() -> Self::Type;
    fn extend(store: &mut Self::Type, iter: impl IntoIterator<Item = T>);
}

#[derive(Debug)]
pub struct Set;

impl<T: Ord> Store<T> for Set {
    type Type = BTreeSet<T>;
    fn create() -> Self::Type {
        Default::default()
    }

    fn extend(store: &mut Self::Type, iter: impl IntoIterator<Item = T>) {
        store.extend(iter);
    }
}

#[derive(Debug)]
pub struct Unique;

impl<T> Store<T> for Unique {
    type Type = Option<T>;

    fn create() -> Self::Type {
        None
    }

    fn extend(store: &mut Self::Type, iter: impl IntoIterator<Item = T>) {
        *store = iter.into_iter().last();
    }
}
