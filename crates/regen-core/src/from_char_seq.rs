use crate::NeverError;
use std::string::FromUtf8Error;

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

#[derive(Debug, Default)]
pub struct StringCharBuilder {
    buf: String,
}

impl FromCharSequence<char> for String {
    type Error = NeverError;
    type Builder = StringCharBuilder;
}

impl FromCharSequenceBuilder<char> for StringCharBuilder {
    type Type = String;
    type Error = NeverError;

    fn append(&mut self, char: char) {
        self.buf.push(char);
    }

    fn build(&self) -> Result<Self::Type, Self::Error> {
        Ok(self.buf.clone())
    }
}

#[derive(Debug, Default)]
pub struct StringU8Builder {
    buf: Vec<u8>,
}

impl FromCharSequence<u8> for String {
    type Error = FromUtf8Error;
    type Builder = StringU8Builder;
}

impl FromCharSequenceBuilder<u8> for StringU8Builder {
    type Type = String;
    type Error = FromUtf8Error;

    fn append(&mut self, char: u8) {
        self.buf.push(char);
    }

    fn build(&self) -> Result<Self::Type, Self::Error> {
        String::from_utf8(self.buf.clone())
    }
}
