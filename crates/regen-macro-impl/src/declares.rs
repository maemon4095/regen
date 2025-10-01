use std::collections::HashMap;

use crate::{expr::PatternChar, pattern::Pattern};
use syn::{punctuated::Punctuated, spanned::Spanned};

#[derive(Debug, Clone)]
pub struct Declares<T: PatternChar> {
    pub variables: HashMap<String, Pattern<T>>,
}

impl<T: PatternChar> Declares<T> {
    pub fn new() -> Self {
        Self {
            variables: Default::default(),
        }
    }

    pub fn merge(&mut self, other: Declares<T>) {
        self.variables.extend(other.variables);
    }
}

impl<T: PatternChar> syn::parse::Parse for Declares<T> {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let assigns = Punctuated::<syn::ExprAssign, syn::Token![;]>::parse_terminated(input)?;
        let variables = assigns
            .into_iter()
            .map(|e| {
                let ident = match *e.left {
                    syn::Expr::Path(e) => e.path.require_ident()?.to_string(),
                    _ => return Err(syn::Error::new(e.span(), "Ident was expected.")),
                };
                let pattern = Pattern::<T>::new(&e.right)?;
                Ok((ident, pattern))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self { variables })
    }
}
