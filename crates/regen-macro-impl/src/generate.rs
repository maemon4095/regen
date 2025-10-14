use std::collections::HashMap;

use crate::{
   field_attibute::FieldAttribute, match_graph::{self, MatchGraph, MatchState}, pattern::{ResolveEnv}, pattern_char::PatternChar, regen_options::RegenOptions, regen_prelude::RegenPrelude, variant_pattern::VariantPattern
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
    let error_type = options.error_type();
    let state_machine_name = resolver.state_machine_type_name(item);
    let state_machine_state_name = resolver.state_machine_state_type_name(item);
    let match_error_type = resolver.match_error_type();

    let root_env = match ResolveEnv::new(&ResolveEnv::empty(), &prelude.declares) {
        Ok(v) => v,
        Err(e) => {
            return e.into_compile_error()        },
    };
    let mut variant_field_attrs = Vec::with_capacity(variants.len());
    let mut builder = match_graph::Builder::new();
    for (assoc, variant) in variants.into_iter().enumerate() {
        let env = match ResolveEnv::new(&root_env, &variant.declares) {
            Ok(v) => v,
            Err(e) => return e.into_compile_error(),
        };
        
        let pattern = match env.resolve(&variant.pattern) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };

        variant_field_attrs.push(variant.field_attrs);
        builder.add(assoc, &pattern);
    }

    let graph = builder.build();
    let errors = match error_check(options, item, prelude, &graph) {
        Ok(v) => v,
        Err(e) => return  e.into_compile_error(),
    };

    let state_variants = graph
        .states()
        .iter()
        .enumerate()
        .map(|(i, e)| generate_state_variant(options, item, &variant_field_attrs, i, e));

    let dead_state_variant = resolver.dead_state_variant_name(); 
    let state_machine_impl = generate_state_machine_impl(options, item, &graph);
    let default_impl = generate_default_impl(options, item, &graph);

    quote! {
        #errors

        impl ::regen::__internal_macro::Parse<#base_type> for #ident {
            type Error = #match_error_type<#error_type>;
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

        #default_impl

        #state_machine_impl
    }
}

fn error_check<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    _prelude: RegenPrelude<T>,
    graph: &MatchGraph<T>
    ) -> syn::Result<TokenStream> {
        let errors = (!options.allow_conflict()).then(|| {
            let conflictions: Vec<_> = graph
            .states()
            .iter()
            .filter(|s| s.assoc().len() > 1)
            .collect();
            let errors = conflictions.iter().map(|state| {
                let mut iter = state.assoc().iter().map(|v| &item.variants[*v].ident).peekable();
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
            quote! {
                #(#errors)*
            }
        }).into_iter();

    Ok(quote! {
        #(#errors)*
    })
}



fn generate_state_variant<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    field_attrs: &[HashMap<String, FieldAttribute>],
    state_index: usize,
    state: &MatchState<T>,
) -> TokenStream {
    let resolver = options.resolver();
    let fields = state.props().iter().map(|prop| {
        let field_name = resolver.state_field_name(prop);
        let attrs = field_attrs[prop.assoc].get(&prop.field).unwrap();
        let ty = resolver.state_field_type(item, attrs, prop);

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

fn generate_default_impl<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum, 
    graph: &MatchGraph<T>
) -> TokenStream {
    let resolver = options.resolver();
    let default_trait = resolver.default_trait();
    let state_machine_name = resolver.state_machine_type_name(item);
    let state_machine_state_name = resolver.state_machine_state_type_name(item);
    let initial_state_variant = resolver.state_variant_name(0);
    let initial_state = &graph.states()[0];

    let field_inits = initial_state.props().iter().map(|prop| {
        let field = resolver.state_field_name(prop);
        quote! {
            #field : #default_trait::default()
        }
    });

    quote! {
        impl #default_trait for #state_machine_name {
            fn default() -> Self {
                Self {
                    state: #state_machine_state_name::#initial_state_variant { 
                        #(#field_inits),*
                    }
                }
            }
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
    let match_error_type = resolver.match_error_type();
    let state_machine_trait = resolver.state_machine_trait();
    let item_name = &item.ident;
    let state_machine_name = resolver.state_machine_type_name(item);

    let advance_impl = generate_advance_impl(options, item, graph);
    let complete_impl = generate_complete_impl(options, item, graph);
    let current_impl = generate_current_impl(options, item, graph);

    quote! {
        impl #state_machine_trait<#base_type> for #state_machine_name {
            type Output = #item_name;
            type Error = #match_error_type<#error_type>;

            #advance_impl
            #complete_impl
            #current_impl
        }
    }
}

fn generate_advance_impl<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum, 
    graph: &MatchGraph<T>,
) -> TokenStream {
    let resolver = options.resolver(); 
    let base_type = resolver.base_type(); 
    let from_char_seq_builder_trait = resolver.from_char_seq_builder_trait();
    let advance_result_type = resolver.advance_result_type(); 
    let default_trait = resolver.default_trait();
    let dead_state = resolver.dead_state_variant_name();
    let state_machine_state_name = resolver.state_machine_state_type_name(item);
    let replace_fn = resolver.replace_fn();

    let states = graph.states();
    let state_type_name = resolver.state_machine_state_type_name(item);
    let state_branches = states.iter().enumerate().map(|(state_index, state)| {
        let variant = resolver.state_variant_name(state_index);
        let fields = state.props().iter().map(|e|{
            let f = resolver.state_field_name(e);
            quote! {
                mut #f
            }
        });

        let branches = state
            .branches()
            .iter()
            .filter_map(|(s, e, t)| t.map(|t| (s, e, t)))
            .map(|(start, end, dst_state_index)| {
                let dst_state = &states[dst_state_index];
                let dst_state_name = resolver.state_variant_name(dst_state_index);

                let introduced_fields_init = dst_state.props().iter().filter(|p| !state.props().contains(p)).map(|prop| {
                    let field = resolver.state_field_name(prop);
                    quote! {
                        let mut #field = #default_trait::default(); 
                    }
                });

                // 一度collectの対象になった後、再びcollectの対象になった場合（前のステートでcollect対象ではなく、なおかつpropに存在する場合）にbuilderをリセットする。
                let re_collect_fields_init = dst_state.collects().iter().filter(|p| state.props().contains(p) && !state.collects().contains(p)).map(|prop| {
                    let field = resolver.state_field_name(prop);
                    quote! {
                        let mut #field = #default_trait::default(); 
                    }
                });

                let fields = dst_state.props().iter().map(|prop| resolver.state_field_name(prop));

                let updates = dst_state.collects().iter().map(|prop| {
                    let field = resolver.state_field_name(prop);
                    quote! { 
                        <_ as #from_char_seq_builder_trait<#base_type>>::append(&mut #field, c);
                    }
                });

                let result = match dst_state.assoc().first() {
                    Some(_) => {
                        quote! { 
                            #advance_result_type::Match(1)
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
                        #(#introduced_fields_init)*

                        #(#re_collect_fields_init)*

                        #(#updates)*

                        self.state = #state_type_name::#dst_state_name {
                            #(#fields),*
                        };
 
                        #result
                    }
                }
            });

        quote! {
            #state_machine_state_name::#variant { #(#fields),* } => {
                match c {
                    #(#branches)*
                    _ => {
                        #advance_result_type::Error
                    }
                }
            }
        }
    });

    quote! {
        fn advance(&mut self, c: #base_type) -> #advance_result_type {
            let state = #replace_fn(&mut self.state, #state_machine_state_name::#dead_state);
            match state {
                #(#state_branches),*
                #state_machine_state_name::#dead_state => {
                    #advance_result_type::Error
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
    let complete_result_type = resolver.complete_result_type(); 
    let dead_state = resolver.dead_state_variant_name();
    let state_machine_state_name = resolver.state_machine_state_type_name(item);
    let replace_fn = resolver.replace_fn();

    let states = graph.states();
    let state_type_name = resolver.state_machine_state_type_name(item);
    let state_branches = states.iter().enumerate().map(|(state_index, state)| {
        let variant = resolver.state_variant_name(state_index);
        let fields  = state.props().iter().map(|e| {
            let f = resolver.state_field_name(e);        
            quote! {
                mut #f
            }
        });

        let result = match state.assoc().first() {
            Some(_) => {
                quote! { 
                    #complete_result_type::Match(1)
                }
            },
            None => {
                quote! {
                    #complete_result_type::Error
                }
            }
        };

        quote! {
            #state_machine_state_name::#variant { #(#fields),* } => {
                self.state = #state_type_name::#dead_state;
                #result
            }
        }
    });

    quote! {
        fn complete(&mut self) -> #complete_result_type {
            let state = #replace_fn(&mut self.state,  #state_machine_state_name::#dead_state);
            match state {
                #(#state_branches),*
                #state_machine_state_name::#dead_state => {
                    #complete_result_type::Error
                }
            }
        }
    }
}

fn generate_current_impl<T: PatternChar>(
    options: &RegenOptions,
    item: &syn::ItemEnum,
    graph: &MatchGraph<T>,
) -> TokenStream {
    let resolver = options.resolver();  
    let match_error_type = resolver.match_error_type();
    let from_char_seq_builder_trait = resolver.from_char_seq_builder_trait(); 
    let dead_state = resolver.dead_state_variant_name();
    let state_machine_state_name = resolver.state_machine_state_type_name(item);
    let item_name = &item.ident;
    let base_type = resolver.base_type();
    let result_type = resolver.result_type();
    let into_trait = resolver.into_trait(); 
    
    let states = graph.states(); 
    let state_branches = states.iter().enumerate().map(|(state_index, state)| {
        let variant = resolver.state_variant_name(state_index);
        let fields  = state.props().iter().map(|e| resolver.state_field_name(e));
        
        let result = match state.assoc().first() {
            Some(&assoc) => {
                let variant = &item.variants[assoc].ident;
                let declares =state.props().iter().filter(|p| p.assoc == assoc).map(|prop| {
                    let field = format_ident!("{}", &prop.field);
                    let state_field = resolver.state_field_name(prop);
                    
                    quote! {
                        let #field = <_ as #from_char_seq_builder_trait<#base_type>>::build(#state_field).map_err(|e| {
                            #match_error_type::Collect(<_ as #into_trait<_>>::into(e))
                        })?;
                    }
                });

                let fields = state.props().iter().filter(|p| p.assoc == assoc).map(|p| format_ident!("{}", &p.field));

                quote! {
                    #(#declares)*
                    #result_type::Ok(
                        #item_name::#variant {
                            #(#fields),*
                        }
                    )
                }
            },
            None => {
                quote! {
                    #result_type::Err(#match_error_type::NotMatched)
                }
            }
        };


        quote! {
            #state_machine_state_name::#variant { #(#fields),* } => {
                #result
            }
        }
    });

    quote! {
        fn current(&self) -> #result_type<Self::Output, Self::Error> {
            match &self.state {
                #(#state_branches),*
                #state_machine_state_name::#dead_state => {
                    #result_type::Err(#match_error_type::NotMatched)
                }
            }
        }
    }
}