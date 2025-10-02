mod from_char_seq;

pub use from_char_seq::{FromCharSequence, FromCharSequenceBuilder};

pub trait Parse<T>: Sized {
    type Error;
    type StateMachine: StateMachine<T, Output = Self, Error = Self::Error>;
}

pub trait StateMachine<T>: Default {
    type Output;
    type Error;
    fn advance(&mut self, c: T) -> AdvanceResult<Self::Output, Self::Error>;
    fn complete(&mut self) -> CompleteResult<Self::Output, Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdvanceResult<T, E> {
    Error(E),
    Partial(usize),
    Rewind(usize),
    Match(T, usize),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompleteResult<T, E> {
    Error(E),
    Match(T, usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum MatchError<E = Box<dyn std::error::Error>> {
    NotMatched,
    Collect(E),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum NeverError {}

impl std::fmt::Display for NeverError {
    fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unreachable!()
    }
}

impl std::error::Error for NeverError {}
