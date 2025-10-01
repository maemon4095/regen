use crate::{
    match_graph::{self, MatchGraph, MatchState},
    pattern_char::PatternChar,
    regen_options::RegenOptions,
    regen_prelude::RegenPrelude,
    resolved_pattern::ResolvedPattern,
    variant_pattern::VariantPattern,
};
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};

pub fn generate_state_machine<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    prelude: RegenPrelude<T>,
    variants: Vec<VariantPattern<T>>,
) -> TokenStream {
    let ident = &item.ident;
    let resolver = options.resolver();
    let base_type = resolver.base_type();
    let state_machine_name = resolver.state_machine_type_name(item);
    let state_machine_state_name = resolver.state_machine_state_type_name(item);

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

    if !options.allow_conflict() {
        let conflictions: Vec<_> = graph
            .states()
            .iter()
            .filter_map(|s| (s.assoc().len() > 1).then_some(s.assoc()))
            .collect();

        if conflictions.len() > 0 {
            let errors = conflictions.iter().map(|vs| {
                let mut iter = vs.iter().map(|v| &item.variants[*v].ident).peekable();
                let mut buf = String::new();

                let first = iter.next().unwrap();
                buf.push('`');
                buf.push_str(&first.to_string());
                buf.push('`');

                while let Some(v) = iter.next() {
                    if iter.peek().is_none() {
                        buf.push_str(" and ");
                    } else {
                        buf.push_str(", ");
                    }

                    buf.push('`');
                    buf.push_str(&v.to_string());
                    buf.push('`');
                }

                syn::Error::new(
                    Span::call_site(),
                    format!("The following patterns are conflicting: {buf}"),
                )
                .into_compile_error()
            });

            return quote! {
                #(#errors)*
            };
        }
    }

    let state_variants = graph
        .states()
        .iter()
        .enumerate()
        .map(|(i, e)| generate_state_variant(options, item, i, e));

    let dead_state_variant = resolver.dead_state_variant_name();

    let state_machine_impl = generate_state_machine_impl(options, item, &graph);

    quote! {
        impl ::regen::__internal_macro::Parse<#base_type> for #ident {
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
            #(#state_variants,)*
            #dead_state_variant
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
    let fields = state.props().iter().map(|prop| {
        let field_name = resolver.state_field_name(&prop);
        let ty = resolver.state_field_type(item, prop);

        quote! {
            #field_name : #ty
        }
    });

    let variant_name = resolver.state_variant_name(state_index);

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
    let base_type = resolver.base_type();
    let error_type = options.error_type(); 
    let state_machine_trait = resolver.state_machine_trait();
    let item_name = &item.ident;
    let state_machine_name = resolver.state_machine_type_name(item);

    let advance_impl = generate_advance_impl(options, item, graph);
    let complete_impl = generate_complete_impl(options, item, graph);

    quote! {
        impl #state_machine_trait<#base_type> for #state_machine_name {
            type Output = #item_name;
            type Error = #error_type;

            #advance_impl
            #complete_impl
        }
    }
}


fn generate_advance_impl<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    graph: &MatchGraph<T>,
) -> TokenStream {
    let resolver = options.resolver(); 
    let error_type = options.error_type();
    let state_machine_error = resolver.state_machine_error_trait(); 
    let from_char_seq_builder_trait = resolver.from_char_seq_builder_trait();
    let advance_result_type = resolver.advance_result_type(); 
    let default_trait = resolver.default_trait();
    let dead_state = resolver.dead_state_variant_name();
    let item_name = &item.ident; 

    let states = graph.states();
    let state_type_name = resolver.state_machine_state_type_name(item);
    let state_branches = states.iter().enumerate().map(|(state_index, state)| {
        let variant = resolver.state_variant_name(state_index);
        let fields = state.props().iter().map(|e| resolver.state_field_name(e));

        let branches = state
            .branches()
            .iter()
            .filter_map(|(s, e, t)| t.map(|t| (s, e, t)))
            .map(|(start, end, dst_state_index)| {
                let dst_state = &states[dst_state_index];
                let dst_state_name = resolver.state_variant_name(dst_state_index);

                let field_inits = dst_state.props().iter().map(|prop| {
                    let field = resolver.state_field_name(prop);
                    if state.props().contains(prop) {
                        quote! {
                            #field
                        }
                    } else {
                        quote! {
                            #field : #default_trait::default()
                        }
                    }
                });

                let updates = dst_state.collects().iter().filter(|p| state.props().contains(p)).map(|prop| {
                    let field = resolver.state_field_name(prop);
                    quote! { 
                        <_ as #from_char_seq_builder_trait>::append(#field, c);
                    }
                });

                let result = match  dst_state.assoc().first() {
                    Some(&assoc) => {
                        let variant = &item.variants[assoc].ident;
                        let field_inits = dst_state.props().iter().filter(|p| p.assoc == assoc).map(|prop| {
                            let field = format_ident!("{}", &prop.field);
                            let state_field = resolver.state_field_name(prop);
                            
                            quote! {
                                #field : <_ as #from_char_seq_builder_trait>::build(#state_field)
                            }
                        });
                        quote! {
                            #advance_result_type::Match(
                                #item_name::#variant {
                                    #(#field_inits),*
                                },
                                1
                            )
                        }
                    },
                    None => {
                        quote! {
                            #advance_result_type::Partial(1)
                        }
                    }
                };

                quote! {
                    #start..#end => {
                        #(#updates)*

                        let result = #result;

                        self.state = #state_type_name::#dst_state_name {
                            #(#field_inits),*
                        };

                        result
                    }
                }
            });

        quote! {
            #variant { #(#fields),* } => {
                match c {
                    #(#branches)*
                    _ => {
                        self.state = #state_type_name::#dead_state;
                        #advance_result_type::Error(<#error_type as #state_machine_error>::not_matched())
                    }
                }
            }
        }
    });

    quote! {
        fn advance(&mut self, c: T) -> #advance_result_type<Self::Output, Self::Error> {
            match self.state {
                #(#state_branches),*
                #dead_state => {
                    #advance_result_type::Error(<#error_type as #state_machine_error>::not_matched())
                }
            }
        }
    }
}

fn generate_complete_impl<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    graph: &MatchGraph<T>,
) -> TokenStream {
    let resolver = options.resolver(); 
    let error_type = options.error_type();
    let state_machine_error = resolver.state_machine_error_trait(); 
    let from_char_seq_builder_trait = resolver.from_char_seq_builder_trait();
    let complete_result_type = resolver.complete_result_type(); 
    let dead_state = resolver.dead_state_variant_name();
    let item_name = &item.ident;

    let states = graph.states();
    let state_type_name = resolver.state_machine_state_type_name(item);
    let state_branches = states.iter().enumerate().map(|(state_index, state)| {
        let variant = resolver.state_variant_name(state_index);
        let fields = state.props().iter().map(|e| resolver.state_field_name(e));
        
        let result = match state.assoc().first() {
            Some(&assoc) => {
                let variant = &item.variants[assoc].ident;
                let field_inits = state.props().iter().filter(|p| p.assoc == assoc).map(|prop| {
                    let field = format_ident!("{}", &prop.field);
                    let state_field = resolver.state_field_name(prop);
                    
                    quote! {
                        #field : <_ as #from_char_seq_builder_trait>::build(#state_field)
                    }
                });
                quote! {
                    #complete_result_type::Match(
                        #item_name::#variant {
                            #(#field_inits),*
                        },
                        1
                    )
                }
            },
            None => {
                quote! {
                    #complete_result_type::Error(<#error_type as #state_machine_error>::not_matched())
                }
            }
        };


        quote! {
            #variant { #(#fields),* } => {
                let result = #result;

                self.state = #state_type_name::#dead_state;

                result
            }
        }
    });

    quote! {
        fn complete(&mut self) -> #complete_result_type<Self::Output, Self::Error> {
            match self.state {
                #(#state_branches),*
                #dead_state => {
                    #complete_result_type::Error(<#error_type as #state_machine_error>::not_matched())
                }
            }
        }
    }
}