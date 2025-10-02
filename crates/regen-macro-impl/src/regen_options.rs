use crate::{base_type::BaseType, match_graph::MatchProp, regen_args::RegenArgs};
use quote::{ToTokens, format_ident, quote};
use syn::{parse_quote, spanned::Spanned as _};

pub struct RegenOptions {
    allow_conflict: bool,
    error_type: syn::Path,
    resolver: PathResolver,
}

impl RegenOptions {
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
    args: RegenArgs,
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

    let resolver = PathResolver::new(args.base_type().clone());
    let error_type = match args.error_type() {
        Some(v) => v.clone(),
        None => {
            let tokens = resolver.default_error_type();
            syn::parse_quote!(#tokens)
        }
    };

    Ok(RegenOptions {
        allow_conflict,
        error_type,
        resolver,
    })
}

pub struct PathResolver {
    base_type: syn::Path,
    regen_macro_lib: syn::Path,
}

impl PathResolver {
    fn new(base_type: BaseType) -> Self {
        let regen_macro_lib = parse_quote!(::regen::__internal_macro);
        let base_type = match base_type {
            BaseType::Char => parse_quote!(#regen_macro_lib::std::char),
            BaseType::U8 => parse_quote!(#regen_macro_lib::std::u8),
            BaseType::U16 => parse_quote!(#regen_macro_lib::std::u16),
            BaseType::U32 => parse_quote!(#regen_macro_lib::std::u32),
            BaseType::U64 => parse_quote!(#regen_macro_lib::std::u64),
        };

        Self {
            base_type,
            regen_macro_lib,
        }
    }

    fn regen_macro_lib(&self) -> impl ToTokens {
        &self.regen_macro_lib
    }

    pub fn base_type(&self) -> impl ToTokens {
        &self.base_type
    }

    pub fn default_trait(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote! {
            #lib::std::Default
        }
    }

    pub fn result_type(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote! {
            #lib::std::Result
        }
    }

    pub fn into_trait(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote! {
            #lib::std::Into
        }
    }

    pub fn replace_fn(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote! {
            #lib::std::replace
        }
    }

    pub fn advance_result_type(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote!(#lib::AdvanceResult)
    }

    pub fn complete_result_type(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote!(#lib::CompleteResult)
    }

    pub fn match_error_type(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote!(#lib::MatchError)
    }

    pub fn default_error_type(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote!(#lib::std::Box<dyn #lib::std::Error>)
    }

    pub fn state_machine_trait(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote!(#lib::StateMachine)
    }

    pub fn from_char_seq_trait(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote!(#lib::FromCharSequence)
    }

    pub fn from_char_seq_builder_trait(&self) -> impl ToTokens {
        let lib = self.regen_macro_lib();
        quote!(#lib::FromCharSequenceBuilder)
    }

    pub fn state_machine_type_name(&self, item: &syn::ItemEnum) -> impl ToTokens {
        format_ident!("__regen_macro_state_machine_{}", item.ident)
    }

    pub fn state_machine_state_type_name(&self, item: &syn::ItemEnum) -> impl ToTokens {
        format_ident!("__regen_macro_state_machine_{}State", item.ident)
    }

    pub fn state_variant_name(&self, state_index: usize) -> impl ToTokens {
        quote::format_ident!("State_{}", state_index)
    }

    pub fn dead_state_variant_name(&self) -> impl ToTokens {
        quote::format_ident!("State_dead")
    }

    pub fn state_field_name(&self, prop: &MatchProp) -> impl ToTokens {
        quote::format_ident!("_{}_{}", prop.assoc, prop.field)
    }

    pub fn state_field_type(&self, item: &syn::ItemEnum, prop: &MatchProp) -> impl ToTokens {
        let base_type = self.base_type();
        let from_char_seq_trait = self.from_char_seq_trait();
        let variant = &item.variants[prop.assoc];
        let ty = variant.fields.iter().enumerate().find_map(|(i, e)| {
            let field = e
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or(i.to_string());

            (field == prop.field).then_some(&e.ty)
        });

        ty.map(|e| {
            quote! {
                <#e as #from_char_seq_trait<#base_type>>::Builder
            }
        })
        .unwrap_or_else(|| {
            syn::Error::new(
                variant.span(),
                format!("no field with name `{}` in the variant", prop.field),
            )
            .to_compile_error()
        })
    }
}
