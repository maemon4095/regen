mod builder;
mod deterministic;
mod nondeterministic;

pub use builder::Builder;
pub use deterministic::{MatchGraph, MatchState};

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct MatchProp {
    pub assoc: usize,
    pub field: String,
}
