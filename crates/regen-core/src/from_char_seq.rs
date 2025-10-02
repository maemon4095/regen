mod string_builder;
mod usize_builder;

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
