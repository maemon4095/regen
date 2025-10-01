use std::cell::LazyCell;

use quote::format_ident;
use syn::parse_quote;

use crate::{base_type::BaseType, match_graph::MatchProp};

pub struct RegenOptions {
    base_type: BaseType,
    allow_conflict: bool,
    error_type: syn::Path,
    resolver: PathResolver,
}

impl RegenOptions {
    pub fn base_type(&self) -> &BaseType {
        &self.base_type
    }

    pub fn allow_conflict(&self) -> bool {
        self.allow_conflict
    }

    pub fn error_type(&self) -> &syn::Path {
        &self.error_type
    }

    pub fn resolver(&self) -> &PathResolver {
        &self.resolver
    }
}

pub fn strip_options(
    item: &mut syn::ItemEnum,
    base_type: BaseType,
) -> Result<RegenOptions, syn::Error> {
    let attrs = &mut item.attrs;
    let mut i = 0;

    let mut allow_conflict = false;
    while i < attrs.len() {
        let Some(ident) = attrs[i].meta.path().get_ident() else {
            i += 1;
            continue;
        };

        if ident == "allow_conflict" {
            let attr = attrs.swap_remove(i);
            attr.meta.require_path_only()?.require_ident()?;
            allow_conflict = true;
            continue;
        }

        i += 1;
    }

    let resolver = PathResolver::new();
    let error_type = resolver.default_match_error_type();

    Ok(RegenOptions {
        base_type,
        allow_conflict,
        error_type,
        resolver,
    })
}

pub struct PathResolver {
    regen_macro_lib: LazyCell<syn::Path>,
}

impl PathResolver {
    fn new() -> Self {
        Self {
            regen_macro_lib: LazyCell::new(|| parse_quote!(::regen::__internal_macro)),
        }
    }

    fn regen_macro_lib(&self) -> &syn::Path {
        &*self.regen_macro_lib
    }

    pub fn advance_result_type(&self) -> syn::Path {
        let lib = self.regen_macro_lib();
        parse_quote!(#lib::AdvanceResult)
    }

    pub fn complete_result_type(&self) -> syn::Path {
        let lib = self.regen_macro_lib();
        parse_quote!(#lib::CompleteResult)
    }

    pub fn default_match_error_type(&self) -> syn::Path {
        let lib = self.regen_macro_lib();
        parse_quote!(#lib::MatchError)
    }

    pub fn state_machine_error_trait(&self) -> syn::Path {
        let lib = self.regen_macro_lib();
        parse_quote!(#lib::StateMachineError)
    }

    pub fn state_machine_trait(&self) -> syn::Path {
        let lib = self.regen_macro_lib();
        parse_quote!(#lib::StateMachine)
    }

    pub fn from_char_seq_trait(&self) -> syn::Path {
        let lib = self.regen_macro_lib();
        parse_quote!(#lib::FromCharSequence)
    }

    pub fn from_char_seq_builder_trait(&self) -> syn::Path {
        let lib = self.regen_macro_lib();
        parse_quote!(#lib::FromCharSequenceBuilder)
    }

    pub fn state_machine_name(&self, item: &syn::ItemEnum) -> syn::Ident {
        format_ident!("__regen_macro_state_machine_{}", item.ident)
    }

    pub fn state_machine_state_name(&self, item: &syn::ItemEnum) -> syn::Ident {
        format_ident!("__regen_macro_state_machine_{}State", item.ident)
    }

    pub fn variant_name(&self, state_index: usize) -> syn::Ident {
        quote::format_ident!("State_{}", state_index)
    }

    pub fn variant_field_name(&self, prop: &MatchProp) -> syn::Ident {
        quote::format_ident!("_{}_{}", prop.assoc, prop.field)
    }
}
