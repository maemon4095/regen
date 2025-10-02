use crate::base_type::BaseType;

pub enum RegenArgs {
    BaseType(BaseType),
    BaseTypeWithError(BaseType, syn::Path),
}

impl RegenArgs {
    pub fn base_type(&self) -> &BaseType {
        match self {
            RegenArgs::BaseType(base_type) => base_type,
            RegenArgs::BaseTypeWithError(base_type, _) => base_type,
        }
    }

    pub fn error_type(&self) -> Option<&syn::Path> {
        match self {
            RegenArgs::BaseType(_) => None,
            RegenArgs::BaseTypeWithError(_, path) => Some(path),
        }
    }
}

impl syn::parse::Parse for RegenArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let base_type = input.parse::<BaseType>()?;

        if !input.peek(syn::Token![,]) {
            return Ok(RegenArgs::BaseType(base_type));
        }
        input.parse::<syn::Token![,]>()?;
        let error_type = input.parse::<syn::Path>().ok();
        match error_type {
            Some(v) => Ok(RegenArgs::BaseTypeWithError(base_type, v)),
            None => Ok(RegenArgs::BaseType(base_type)),
        }
    }
}
