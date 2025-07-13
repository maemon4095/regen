pub enum RegenOptions {
    BaseType(BaseType),
}

impl RegenOptions {
    pub fn base_type(&self) -> &BaseType {
        match self {
            RegenOptions::BaseType(base_type) => base_type,
        }
    }
}

impl syn::parse::Parse for RegenOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(RegenOptions::BaseType(input.parse::<BaseType>()?))
    }
}

pub enum BaseType {
    Char,
    U8,
    U16,
    U32,
    U64,
}

impl quote::ToTokens for BaseType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            BaseType::Char => kw::char::default().to_tokens(tokens),
            BaseType::U8 => kw::u8::default().to_tokens(tokens),
            BaseType::U16 => kw::u16::default().to_tokens(tokens),
            BaseType::U32 => kw::u32::default().to_tokens(tokens),
            BaseType::U64 => kw::u64::default().to_tokens(tokens),
        }
    }
}

mod kw {
    syn::custom_keyword!(char);
    syn::custom_keyword!(u8);
    syn::custom_keyword!(u16);
    syn::custom_keyword!(u32);
    syn::custom_keyword!(u64);
}

impl syn::parse::Parse for BaseType {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(kw::char) {
            input.parse::<kw::char>().unwrap();
            return Ok(BaseType::Char);
        }

        if input.peek(kw::u8) {
            input.parse::<kw::u8>().unwrap();
            return Ok(BaseType::U8);
        }

        if input.peek(kw::u16) {
            input.parse::<kw::u16>().unwrap();
            return Ok(BaseType::U16);
        }

        if input.peek(kw::u32) {
            input.parse::<kw::u32>().unwrap();
            return Ok(BaseType::U32);
        }

        if input.peek(kw::u64) {
            input.parse::<kw::u64>().unwrap();
            return Ok(BaseType::U64);
        }

        Err(input.error("expected u8, u16, u32, or u64."))
    }
}
