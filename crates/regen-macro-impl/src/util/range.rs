use std::ops::{Bound, RangeBounds};

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

pub fn is_range_empty(start: Bound<usize>, end: Bound<usize>) -> bool {
    match (start, end) {
        (Bound::Included(min), Bound::Included(max)) => min > max,
        (Bound::Included(min), Bound::Excluded(max)) => min >= max,
        (Bound::Included(_), Bound::Unbounded) => false,
        (Bound::Excluded(min), Bound::Included(max)) => min >= max,
        (Bound::Excluded(min), Bound::Excluded(max)) => {
            max.checked_sub(min).map(|d| d > 1).unwrap_or(false)
        }
        (Bound::Excluded(min), Bound::Unbounded) => min == usize::MAX,
        (Bound::Unbounded, Bound::Included(_)) => false,
        (Bound::Unbounded, Bound::Excluded(max)) => max == 0,
        (Bound::Unbounded, Bound::Unbounded) => false,
    }
}
