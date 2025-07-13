mod expr;
mod generate;
mod match_tree;
mod pattern;
mod regen_options;

use proc_macro2::TokenStream;
use quote::quote;
use regen_options::RegenOptions;
use syn::spanned::Spanned;

use crate::{generate::generate_state_machine_impl, pattern::Pattern};

pub fn regen(attr: TokenStream, body: TokenStream) -> TokenStream {
    let options: RegenOptions = match syn::parse2(attr) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let mut item: syn::ItemEnum = match syn::parse2(body) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let pairs = match strip_patterns(&mut item) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let state_machine_impl = generate_state_machine_impl(&item, &options, pairs);

    quote! {
        #item
        #state_machine_impl
    }
}

fn strip_patterns(item: &mut syn::ItemEnum) -> Result<Vec<VariantPatternPair>, syn::Error> {
    let mut buf = Vec::new();
    for v in &mut item.variants {
        let mut attrs = v.attrs.extract_if(.., |a| {
            let Some(ident) = a.meta.path().get_ident() else {
                return false;
            };

            ident == "pattern"
        });

        let Some(attr) = attrs.next() else {
            continue;
        };

        if let Some(a) = attrs.next() {
            return Err(syn::Error::new(a.span(), "Duplicated pattern attributes."));
        }

        drop(attrs);

        let name_value = attr.meta.require_name_value()?;
        let pattern = Pattern::new(&name_value.value)?;
        buf.push(VariantPatternPair {
            pattern,
            variant: v.clone(),
        });
    }
    Ok(buf)
}

struct VariantPatternPair {
    pub pattern: Pattern,
    pub variant: syn::Variant,
}

#[cfg(test)]
mod test {
    pub use super::*;

    #[test]
    fn test() {
        let attr: TokenStream = syn::parse_quote! {
            u8
        };

        let body: TokenStream = syn::parse_quote! {
            #[derive(Default)]
            pub enum Test {
                #[pattern = repeat!("a", 1..) + repeat!("a" | "b") + char::class]
                #[default]
                A,

            }
        };

        let tokens = regen(attr, body);

        let file: syn::File = syn::parse2(tokens).unwrap();

        println!("{}", prettyplease::unparse(&file));
    }
}
