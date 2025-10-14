use crate::{
    match_graph::{BuildContext, Builder, MatchPattern, StateId},
    pattern::{Pattern, PatternAtom, PatternKind, PatternTag, ResolvedPatternTag},
    pattern_char::PatternChar,
};
use syn::spanned::Spanned as _;

// PatternSeq ::= array | bstr | str
#[derive(Debug, Clone)]
pub struct PatternSeq<T: PatternChar, K: PatternKind = PatternTag> {
    pub patterns: Vec<K::Pattern<T>>,
}

impl<T: PatternChar> PatternSeq<T> {
    pub fn from_str(str: &syn::LitStr) -> syn::Result<Self> {
        let patterns = str
            .value()
            .chars()
            .map(|e| {
                T::try_from_char(e)
                    .map(PatternAtom::Primitive)
                    .map(Pattern::Atom)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|m| syn::Error::new(str.span(), m))?;

        if patterns.is_empty() {
            Err(syn::Error::new(
                str.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { patterns })
        }
    }

    pub fn from_bstr(str: &syn::LitByteStr) -> syn::Result<Self> {
        let patterns = str
            .value()
            .iter()
            .map(|&e| {
                T::try_from_u8(e)
                    .map(PatternAtom::Primitive)
                    .map(Pattern::Atom)
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|m| syn::Error::new(str.span(), m))?;

        if patterns.is_empty() {
            Err(syn::Error::new(
                str.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { patterns })
        }
    }

    pub fn from_array(arr: &syn::ExprArray) -> syn::Result<Self> {
        let patterns = arr
            .elems
            .iter()
            .map(|e| Pattern::new(e))
            .collect::<Result<Vec<_>, _>>()?;

        if patterns.is_empty() {
            Err(syn::Error::new(
                arr.span(),
                "Sequence pattern must not be empty.",
            ))
        } else {
            Ok(PatternSeq { patterns })
        }
    }
}

impl<T: PatternChar> MatchPattern<T> for PatternSeq<T, ResolvedPatternTag> {
    fn insert(
        &self,
        builder: &mut Builder<T>,
        context: &mut BuildContext,
        from: StateId,
    ) -> StateId {
        let mut state = from;
        for p in &self.patterns {
            state = p.insert(builder, context, state);
        }
        state
    }
}
