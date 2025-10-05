use std::num::ParseIntError;

use regen::{regen, FromCharSequence, FromCharSequenceBuilder};

#[derive(Debug, PartialEq, Eq)]
#[regen(u8)]
enum PartialMatch {
    #[pattern = collect!(_x <- b"ab")]
    X { _x: String },
    #[pattern = collect!(_x <- [b"a"; ..])]
    Y { _x: String },
    #[pattern = collect!(_x <- [b'a', b"c"])]
    Z { _x: String }
}

#[derive(Debug, PartialEq, Eq)]
#[regen(char, ParseIntError)]
enum DecimalUsize {
    #[pattern = collect!(_num <- ['0'..='9'; 1..])]
    Decimal {
        _num: usize
    }
}

#[derive(Debug, PartialEq, Eq)]
#[regen(char)]
enum Complex {
    #[pattern = collect!(_digits <- ['0'..='9'; 1..]) 
            | ("0" + collect!(_radix <- "b") + collect!(_digits <- [('0' | '1'); 1..]))
            | ("0" + collect!(_radix <- "o") + collect!(_digits <- ['0'..='7'; 1..]))
            | ("0" + collect!(_radix <- "x") + collect!(_digits <- [('0'..='9') | ('A'..='F') | ('a'..='f'); 1..]))]
    Digits { _radix: Radix, _digits: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Radix {
    Binary,
    Decimal,
    Octal,
    Hexadecimal
}

impl FromCharSequence<char> for Radix {
    type Error = RadixError;
    type Builder = RadixBuilder;
}

#[derive(Default)]
struct RadixBuilder { 
    radix: Option<Result<Radix, RadixError>>
}

impl FromCharSequenceBuilder<char> for RadixBuilder {
    type Type = Radix;
    type Error = RadixError;

    fn append(&mut self, char: char) {
        if self.radix.is_some() {
            return;
        }
        match char {
            'b' => self.radix = Some(Ok(Radix::Binary)),
            'o' => self.radix = Some(Ok(Radix::Octal)),
            'x' => self.radix = Some(Ok(Radix::Hexadecimal)),
            _ => self.radix = Some(Err(RadixError))
        }
    }

    fn build(&self) -> Result<Self::Type, Self::Error> {
        self.radix.clone().transpose().map(|x| x.unwrap_or(Radix::Decimal))
    }
}
#[derive(Debug, Clone)]
struct RadixError;

impl std::fmt::Display for RadixError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "`x`, `b` or `o` was expected.")
    }
}
impl std::error::Error for RadixError {}


#[derive(Debug, PartialEq)]
#[regen(char)]
enum HexUsize {
    #[pattern = collect!(_num <- [('0'..='9') | ('A'..='F'); 1..])]
    Hex {
        #[builder = UsizeHexBuilder]
        _num: usize
    }
}

#[derive(Debug, Default)]
struct UsizeHexBuilder {
    digits: String
}

impl FromCharSequenceBuilder<char> for UsizeHexBuilder {
    type Type = usize;
    type Error = ParseIntError;

    fn append(&mut self, char: char) {
        self.digits.push(char);
    }

    fn build(&self) -> Result<Self::Type, Self::Error> {
        usize::from_str_radix(&self.digits, 16)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use regen::{AdvanceResult, MatchError, Parse, StateMachine};

    #[test]
    fn test_partial_match_x() {
        let mut machine = <PartialMatch as Parse<u8>>::StateMachine::default();

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(
                    v,
                    PartialMatch::Y {
                        _x: String::from("a")
                    }
                );
                assert_eq!(c, 1)
            }
            _ => unreachable!(),
        }

        let r = machine.advance(b'b');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(
                    v,
                    PartialMatch::X {
                        _x: String::from("ab")
                    }
                );
                assert_eq!(c, 1)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_partial_match_y() {
        let mut machine = <PartialMatch as Parse<u8>>::StateMachine::default();

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(
                    v,
                    PartialMatch::Y {
                        _x: String::from("a")
                    }
                );
                assert_eq!(c, 1)
            }
            _ => unreachable!(),
        }

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(
                    v,
                    PartialMatch::Y {
                        _x: String::from("aa")
                    }
                );
                assert_eq!(c, 1)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_complex_decimal() {
        let mut machine = <Complex as Parse<char>>::StateMachine::default();

        let r = machine.advance('0');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(v, Complex::Digits { _radix: Radix::Decimal, _digits: format!("0") });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }

        let r = machine.advance('1');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(v, Complex::Digits { _radix: Radix::Decimal, _digits: format!("01") });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }

        let r = machine.advance('a');
        match r {
            AdvanceResult::Error => {
                let e = machine.current().unwrap_err();
                assert!(matches!(e, MatchError::NotMatched));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_complex_hex() {
        let mut machine = <Complex as Parse<char>>::StateMachine::default();

        let r = machine.advance('0');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(v, Complex::Digits { _radix: Radix::Decimal, _digits: format!("0") });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }

        let r = machine.advance('x');
        match r {
            AdvanceResult::Partial(c) => {
                let e = machine.current().unwrap_err(); 
                assert!(matches!(e, MatchError::NotMatched));
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }

        let r = machine.advance('F');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(v, Complex::Digits { _radix: Radix::Hexadecimal, _digits: format!("F") });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }

        let r = machine.advance('E');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(v, Complex::Digits { _radix: Radix::Hexadecimal, _digits: format!("FE") });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_hex_usize() {
        let mut machine = <HexUsize as Parse<char>>::StateMachine::default();

        let r = machine.advance('F');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(v, HexUsize::Hex { _num: 0xF });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }

        let r = machine.advance('E');
        match r {
            AdvanceResult::Match(c) => {
                let v = machine.current().unwrap();
                assert_eq!(v, HexUsize::Hex { _num: 0xFE });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }
    }
}
