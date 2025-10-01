use quote::ToTokens;

use crate::util::Discrete;

pub trait PatternChar: Ord + Copy + Eq + ToTokens + Discrete {
    fn try_from_lit(lit: &syn::Lit) -> Result<Self, syn::Error>;
    fn try_from_char(c: char) -> Result<Self, &'static str>;
    fn try_from_u8(b: u8) -> Result<Self, &'static str>;
}

impl PatternChar for char {
    fn try_from_lit(lit: &syn::Lit) -> Result<Self, syn::Error> {
        match lit {
            syn::Lit::Char(c) => Ok(c.value()),
            _ => Err(syn::Error::new(lit.span(), "char literal was expected.")),
        }
    }

    fn try_from_char(c: char) -> Result<Self, &'static str> {
        Ok(c)
    }

    fn try_from_u8(_: u8) -> Result<Self, &'static str> {
        Err("char literal was expected.")
    }
}

macro_rules! impl_pattern_primitive {
    (@int $($ty: ty),*) => {
        $(
            impl PatternChar for $ty {
                fn try_from_lit(lit: &syn::Lit) -> Result<Self, syn::Error> {
                    let v = match lit {
                        syn::Lit::Byte(c) => c.value().into(),
                        syn::Lit::Int(c) => c.base10_parse()?,
                        _ =>return Err(syn::Error::new(lit.span(), concat!(stringify!($ty), " literal was expected."))),
                    };
                    Ok(v)
                }

                fn try_from_char(_: char) -> Result<Self, &'static str> {
                    Err(concat!(stringify!($ty), " literal was expected."))
                }

                fn try_from_u8(b: u8) -> Result<Self, &'static str> {
                    Ok(b.into())
                }
            }
        )*
    };
}

impl_pattern_primitive!(@int usize, u8, u16, u32, u64);
