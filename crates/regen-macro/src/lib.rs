use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn regen(attr: TokenStream, body: TokenStream) -> TokenStream {
    regen_macro_impl::regen(attr.into(), body.into()).into()
}
