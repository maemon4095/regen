mod base_type;
mod declares;
mod eval;
mod field_attibute;
mod generate;
mod linkedlist;
mod match_graph;
mod pattern;
mod pattern_char;
mod regen_args;
mod regen_options;
mod regen_prelude;
mod util;
mod variant_pattern;

use base_type::BaseType;
use generate::generate_state_machine;
use pattern_char::PatternChar;
use proc_macro2::TokenStream;
use quote::quote;
use regen_args::RegenArgs;
use regen_options::strip_options;
use regen_prelude::strip_prelude;
use variant_pattern::strip_variant_attrs;

pub fn regen(attr: TokenStream, body: TokenStream) -> TokenStream {
    let options: RegenArgs = match syn::parse2(attr) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let item: syn::ItemEnum = match syn::parse2(body) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    match options.base_type() {
        BaseType::Char => gen_impl::<char>(options, item),
        BaseType::U8 => gen_impl::<u8>(options, item),
        BaseType::U16 => gen_impl::<u16>(options, item),
        BaseType::U32 => gen_impl::<u32>(options, item),
        BaseType::U64 => gen_impl::<u64>(options, item),
    }
}

fn gen_impl<T: PatternChar>(args: RegenArgs, mut item: syn::ItemEnum) -> TokenStream {
    let options = match strip_options(&mut item, args) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let prelude = match strip_prelude(&mut item) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let variants = match strip_variant_attrs(&mut item) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let state_machine = generate_state_machine::<T>(&options, &item, prelude, variants);

    quote! {
        #item
        #state_machine
    }
}

#[cfg(test)]
mod test {
    pub use super::*;

    #[test]
    fn test_conflict() {
        let attr: TokenStream = syn::parse_quote! {
            u8
        };

        let body: TokenStream = syn::parse_quote! {
            pub enum Test {
                #[pattern = b"abc"]
                #[default]
                A { x: String },
                #[pattern = b"abc"]
                #[default]
                B { x: String },
            }
        };

        let tokens = regen(attr, body);
        let file: syn::File = syn::parse2(tokens).unwrap();

        let x = file.items.iter().find_map(|e| match e {
            syn::Item::Macro(item_macro) => {
                if item_macro.mac.path == syn::parse_quote!(::core::compile_error) {
                    item_macro
                        .mac
                        .parse_body::<syn::LitStr>()
                        .ok()
                        .map(|e| e.value())
                } else {
                    None
                }
            }
            _ => None,
        });

        assert_eq!(
            x,
            Some(String::from(
                "The following patterns are conflicting: `A` and `B`"
            ))
        );
    }
}
