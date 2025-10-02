pub use regen_core::*;
pub use regen_macro::regen;

pub mod __internal_macro {
    pub use regen_core::*;
    pub mod std {
        pub use Box;
        pub use Default;
        pub use Into;
        pub use Result;
        pub use std::error::Error;
        pub use std::mem::replace;
        pub use {char, u8, u16, u32, u64};
    }
}
