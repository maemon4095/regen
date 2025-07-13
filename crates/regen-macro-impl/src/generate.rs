use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::{VariantPatternPair, regen_options::RegenOptions};

pub fn generate_state_machine_impl(
    item: &syn::ItemEnum,
    options: &RegenOptions,
    pairs: Vec<VariantPatternPair>,
) -> TokenStream {
    let base_ty = options.base_type();
    let ident = &item.ident;
    let state_machine_ident = format_ident!("__regen_macro_state_machine_{}", ident);

    quote! {
        impl ::regen::__internal_macro::Parse<#base_ty> for #ident {
            type StateMachine = #state_machine_ident;
        }

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        enum #state_machine_ident {

        }
    }
}
