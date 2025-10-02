use regen::regen;

#[derive(Debug, PartialEq, Eq)]
#[regen(u8)]
enum A {
    #[pattern = collect!(_x, b"ab")]
    X { _x: String },
    #[pattern = collect!(_x, [b"a"; ..])]
    Y { _x: String },
}

#[cfg(test)]
mod test {
    use regen::{AdvanceResult, Parse, StateMachine};

    use super::*;

    #[test]
    fn test_match_x() {
        let mut machine = <A as Parse<u8>>::StateMachine::default();

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(v, i) => {
                assert_eq!(
                    v,
                    A::Y {
                        _x: String::from("a")
                    }
                );
                assert_eq!(i, 1)
            }
            _ => unreachable!(),
        }

        let r = machine.advance(b'b');
        match r {
            AdvanceResult::Match(v, i) => {
                assert_eq!(
                    v,
                    A::X {
                        _x: String::from("ab")
                    }
                );
                assert_eq!(i, 1)
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn test_match_y() {
        let mut machine = <A as Parse<u8>>::StateMachine::default();

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(v, i) => {
                assert_eq!(
                    v,
                    A::Y {
                        _x: String::from("a")
                    }
                );
                assert_eq!(i, 1)
            }
            _ => unreachable!(),
        }

        let r = machine.advance(b'a');
        match r {
            AdvanceResult::Match(v, i) => {
                assert_eq!(
                    v,
                    A::Y {
                        _x: String::from("aa")
                    }
                );
                assert_eq!(i, 1)
            }
            _ => unreachable!(),
        }
    }
}
