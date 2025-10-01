pub trait Discrete: Sized {
    fn next_up(&self) -> Option<Self>;
}

macro_rules! impl_discete {
    (@int $($ty: ty),*) => {
        $(
            impl Discrete for $ty {
                fn next_up(&self) -> Option<Self> {
                    self.checked_add(1)
                }
            }
        )*
    };

    (@fp $($ty: ty),*) => {
        $(
            impl Discrete for $ty {
                fn next_up(&self) -> Option<Self> {
                    if self.is_sign_positive() && self.is_infinite() {
                        None
                    } else {
                        Some(Self::next_up(*self))
                    }
                }
            }
        )*
    };
}

impl_discete!(@int usize, u64, u32, u16, u8);
impl_discete!(@fp f64, f32);

impl Discrete for char {
    fn next_up(&self) -> Option<Self> {
        let n = (*self as u32).checked_add(1)?;
        char::from_u32(n)
    }
}
