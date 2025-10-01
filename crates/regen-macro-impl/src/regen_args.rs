use crate::base_type::BaseType;

pub enum RegenArgs {
    BaseType(BaseType),
}

impl RegenArgs {
    pub fn base_type(&self) -> &BaseType {
        match self {
            RegenArgs::BaseType(base_type) => base_type,
        }
    }
}

impl syn::parse::Parse for RegenArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(RegenArgs::BaseType(input.parse::<BaseType>()?))
    }
}
