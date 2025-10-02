use crate::{FromCharSequence, FromCharSequenceBuilder};
use std::num::ParseIntError;

impl FromCharSequence<char> for usize {
    type Error = ParseIntError;
    type Builder = UsizeCharBuilder;
}

#[derive(Debug, Default)]
pub struct UsizeCharBuilder {
    buf: String,
}

impl FromCharSequenceBuilder<char> for UsizeCharBuilder {
    type Type = usize;
    type Error = ParseIntError;

    fn append(&mut self, char: char) {
        self.buf.push(char);
    }

    fn build(&self) -> Result<Self::Type, Self::Error> {
        self.buf.parse()
    }
}
