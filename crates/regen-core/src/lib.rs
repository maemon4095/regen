pub trait Parse<T>: Sized {
    type StateMachine: StateMachine<T, Output = Self>;
}

pub trait StateMachine<T>: Default {
    type Output;
    fn advance(&mut self, c: T) -> AdvanceResult<Self::Output>;
    fn complete(&mut self) -> CompleteResult<Self::Output>;
}

pub enum AdvanceResult<T> {
    Error,
    Partial(usize),
    Done(T, usize),
}

pub enum CompleteResult<T> {
    Error,
    Done(T, usize),
}

pub trait CharClass<T> {
    fn contains(&self, c: &T) -> bool;
}
