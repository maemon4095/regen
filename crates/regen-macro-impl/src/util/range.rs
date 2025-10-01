use std::ops::RangeBounds;

use crate::util::Discrete;

pub fn to_ropen<T: Discrete + Clone>(r: impl RangeBounds<T>) -> (Option<T>, Option<T>) {
    let start = match r.start_bound() {
        std::ops::Bound::Included(v) => Some(v.clone()),
        std::ops::Bound::Excluded(v) => v.next_up(),
        std::ops::Bound::Unbounded => None,
    };

    let end = match r.end_bound() {
        std::ops::Bound::Included(v) => v.next_up(),
        std::ops::Bound::Excluded(v) => Some(v.clone()),
        std::ops::Bound::Unbounded => None,
    };

    (start, end)
}
