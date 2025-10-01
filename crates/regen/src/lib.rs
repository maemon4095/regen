pub use regen_core::*;
pub use regen_macro::regen;

pub mod __internal_macro {
    pub use regen_core::*;
    pub mod std {
        pub use Default;
    }
}
