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

pub enum AdvanceResult<T, E> {
    Error(E),
    Partial(usize),
    Done(T, usize),
}

pub enum CompleteResult<T, E> {
    Error(E),
    Done(T, usize),
}

pub trait FromCharSequence<T> {
    type Error;
    type Builder: FromCharSequenceBuilder<T, Error = Self::Error>;
}

pub trait FromCharSequenceBuilder<T>: Default {
    type Type: FromCharSequence<T>;
    type Error;

    fn append(&mut self, char: T);
    fn build(&self) -> Result<Self::Type, Self::Error>;
}

pub trait StateMachineError {
    fn not_matched() -> Self;
}

pub enum MatchError {
    NotMatched,
    Collect(Box<dyn std::error::Error>),
}

impl StateMachineError for MatchError {
    fn not_matched() -> Self {
        Self::NotMatched
    }
}
