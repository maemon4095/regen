use regen::regen;

#[derive(Debug)]
#[regen(u8)]
enum A {
    #[pattern = collect!(x, b"ab")]
    X { x: String },
}

#[cfg(test)]
mod test {
    use regen::{Parse, StateMachine};

    use super::*;

    #[test]
    fn test() {
        let mut machine = <A as Parse<u8>>::StateMachine::default();

        let r = machine.advance(b'a');
        dbg!(&r);

        let r = machine.advance(b'b');
        dbg!(&r);
    }
}
