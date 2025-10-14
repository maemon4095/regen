use std::fmt::Debug;

pub struct SortedVec<T: Ord> {
    buf: Vec<T>,
}

impl<T: Ord> SortedVec<T> {
    pub fn new() -> Self {
        Self { buf: Vec::new() }
    }
}

impl<T: Ord + Debug> Debug for SortedVec<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(&self.buf).finish()
    }
}

impl<T: Ord> Default for SortedVec<T> {
    fn default() -> Self {
        Self {
            buf: Default::default(),
        }
    }
}

impl<T: Ord> FromIterator<T> for SortedVec<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut buf: Vec<T> = iter.into_iter().collect();
        buf.sort();
        Self { buf }
    }
}

impl<T: Ord> std::ops::Deref for SortedVec<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}
