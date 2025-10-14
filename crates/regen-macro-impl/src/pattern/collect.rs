use crate::{
    match_graph::{BuildContext, Builder, MatchPattern, StateId},
    pattern::{Pattern, PatternKind, PatternTag, ResolvedPatternTag},
    pattern_char::PatternChar,
};
use syn::parse::Parse;

// PatternCollect ::= "collect!(" + path  + "," + pattern ")"
#[derive(Debug, Clone)]
pub struct PatternCollect<T: PatternChar, K: PatternKind = PatternTag> {
    pub field: String,
    pub pattern: K::Pattern<T>,
}

impl<T: PatternChar> PatternCollect<T> {
    pub fn from_mac(mac: &syn::Macro) -> syn::Result<Self> {
        let e = mac.parse_body_with(parser_fn(move |input| {
            let member = syn::Member::parse(input)?;
            let _ = <syn::Token![<-]>::parse(input)?;
            let e = syn::Expr::parse(input)?;
            let pattern = Pattern::new(&e)?;

            if !input.is_empty() {
                return Err(input.error("Unexpected arguments of `collect!`."));
            }

            let field = match member {
                syn::Member::Named(ident) => ident.to_string(),
                syn::Member::Unnamed(index) => index.index.to_string(),
            };

            Ok(PatternCollect { field, pattern })
        }))?;

        Ok(e)
    }
}

impl<T: PatternChar> MatchPattern<T> for PatternCollect<T, ResolvedPatternTag> {
    fn insert(
        &self,
        builder: &mut Builder<T>,
        context: &mut BuildContext,
        from: StateId,
    ) -> StateId {
        builder.insert_collect(context, from, &self.field, &self.pattern)
    }
}

fn parser_fn<F, T>(f: F) -> F
where
    F: for<'a> FnOnce(&'a syn::parse::ParseBuffer<'a>) -> T,
{
    f
}
