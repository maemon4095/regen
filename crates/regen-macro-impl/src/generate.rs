use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

use crate::{
    expr::PatternChar,
    match_graph::{self, MatchGraph, MatchState},
    regen_options::RegenOptions,
    regen_prelude::RegenPrelude,
    resolved_pattern::ResolvedPattern,
    variant_pattern::VariantPattern,
};

pub fn generate_state_machine<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    prelude: RegenPrelude<T>,
    variants: Vec<VariantPattern<T>>,
) -> TokenStream {
    let base_ty = options.base_type();
    let ident = &item.ident;
    let resolver = options.resolver();
    let state_machine_name = resolver.state_machine_name(item);
    let state_machine_state_name = resolver.state_machine_state_name(item);

    let mut builder = match_graph::Builder::new();

    for (assoc, variant) in variants.into_iter().enumerate() {
        let mut declares = prelude.declares.clone();
        declares.merge(variant.declares);

        let pattern = match ResolvedPattern::resolve(&declares, variant.pattern) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };
        builder.add(assoc, pattern);
    }

    let graph = builder.build();

    let state_variants = graph
        .states()
        .iter()
        .enumerate()
        .map(|(i, e)| generate_state_variant(options, item, i, e));

    let state_machine_impl = generate_state_machine_impl(options, item, &graph);

    quote! {
        impl ::regen::__internal_macro::Parse<#base_ty> for #ident {
            type StateMachine = #state_machine_name;
        }

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        struct #state_machine_name {
            state: #state_machine_state_name
        }

        #[doc(hidden)]
        #[allow(non_camel_case_types)]
        enum #state_machine_state_name {
            #(#state_variants),*
        }

        #state_machine_impl
    }
}

fn generate_state_variant<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    state_index: usize,
    state: &MatchState<T>,
) -> TokenStream {
    let resolver = options.resolver();
    let base_type = options.base_type();
    let from_char_seq_trait = resolver.from_char_seq_trait();
    let fields = state.props().map(|prop| {
        let field_name = resolver.variant_field_name(&prop);
        let variant = &item.variants[prop.assoc];
        let ty = variant.fields.iter().enumerate().find_map(|(i, e)| {
            let field = e
                .ident
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or(i.to_string());

            (field == prop.field).then_some(&e.ty)
        });

        let ty = ty
            .map(|e| {
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
            });

        quote! {
            #field_name : #ty
        }
    });

    let variant_name = resolver.variant_name(state_index);

    quote! {
        #variant_name {
            #(
                #fields
            ),*
        }
    }
}

fn generate_state_machine_impl<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    graph: &MatchGraph<T>,
) -> TokenStream {
    let resolver = options.resolver();
    let base_type = options.base_type();
    let error_type = options.error_type();
    let state_machine_error = resolver.state_machine_error_trait();
    let state_machine_trait = resolver.state_machine_trait();

    let item_name = &item.ident;
    let state_machine_name = resolver.state_machine_name(item);
    let state_branches = graph
        .states()
        .iter()
        .enumerate()
        .map(|(state_index, state)| {
            let variant = resolver.variant_name(state_index);
            let fields = state.props().map(|e| resolver.variant_field_name(e));

            let branches = state
                .branches()
                .iter()
                .filter_map(|(s, e, t)| t.map(|t| (s, e, t)))
                .map(|(start, end, to)| {
                    quote! {}
                });

            quote! {
                #variant { #(#fields),* } => {
                    match c {
                        #(#branches)*
                        _ => {
                            <#error_type as #state_machine_error>::not_matched()
                        }
                    }
                }
            }
        });

    quote! {
        impl #state_machine_trait<#base_type> for #state_machine_name {
            type Output = #item_name;
            type Error = #error_type;

            fn advance(&mut self, c: T) -> AdvanceResult<Self::Output, Self::Error> {
                match self.state {
                    #(#state_branches),*
                }
            }
            fn complete(&mut self) -> CompleteResult<Self::Output, Self::Error> {

            }
        }
    }
}
