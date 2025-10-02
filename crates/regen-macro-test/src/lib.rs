use std::num::ParseIntError;

use regen::regen;

#[derive(Debug, PartialEq, Eq)]
#[regen(u8)]
enum A {
    #[pattern = collect!(_x <- b"ab")]
    X { _x: String },
    #[pattern = collect!(_x <- [b"a"; ..])]
    Y { _x: String },
}

#[derive(Debug, PartialEq, Eq)]
#[regen(char, ParseIntError)]
enum B {
    #[pattern = collect!(_x <- ['0'..'9'; 1..])]
    Z { _x: usize },
}

#[cfg(test)]
mod test {
    use super::*;
    use regen::{AdvanceResult, MatchError, Parse, StateMachine};

    #[test]
    fn test_match_x() {
        let mut machine = <A as Parse<u8>>::StateMachine::default();

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(v, c) => {
                assert_eq!(
                    v,
                    A::Y {
                        _x: String::from("a")
                    }
                );
                assert_eq!(c, 1)
            }
            _ => unreachable!(),
        }

        let r = machine.advance(b'b');
        match r {
            AdvanceResult::Match(v, c) => {
                assert_eq!(
                    v,
                    A::X {
                        _x: String::from("ab")
                    }
                );
                assert_eq!(c, 1)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_match_y() {
        let mut machine = <A as Parse<u8>>::StateMachine::default();

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(v, c) => {
                assert_eq!(
                    v,
                    A::Y {
                        _x: String::from("a")
                    }
                );
                assert_eq!(c, 1)
            }
            _ => unreachable!(),
        }

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(v, c) => {
                assert_eq!(
                    v,
                    A::Y {
                        _x: String::from("aa")
                    }
                );
                assert_eq!(c, 1)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_match_z() {
        let mut machine = <B as Parse<char>>::StateMachine::default();

        let r = machine.advance('0');
        match r {
            AdvanceResult::Match(v, c) => {
                assert_eq!(v, B::Z { _x: 0 });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }

        let r = machine.advance('1');
        match r {
            AdvanceResult::Match(v, c) => {
                assert_eq!(v, B::Z { _x: 1 });
                assert_eq!(c, 1);
            }
            _ => unreachable!(),
        }

        let r = machine.advance('a');
        match r {
            AdvanceResult::Error(e) => {
                assert_eq!(e, MatchError::NotMatched);
            }
            _ => unreachable!(),
        }
    }
}
