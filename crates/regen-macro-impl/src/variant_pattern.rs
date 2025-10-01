use crate::{declares::Declares, pattern::Pattern, pattern_char::PatternChar};
use syn::spanned::Spanned;

pub struct VariantPattern<T: PatternChar> {
    pub pattern: Pattern<T>,
    pub declares: Declares<T>,
}

pub fn strip_variant_attrs<T: PatternChar>(
    item: &mut syn::ItemEnum,
) -> Result<Vec<VariantPattern<T>>, syn::Error> {
    let mut buf = Vec::new();
    for v in &mut item.variants {
        let Some(pattern) = strip_variant_patterns(v)? else {
            continue;
        };

        let declares = strip_variant_declares(v)?;

        buf.push(VariantPattern { pattern, declares });
    }
    Ok(buf)
}

fn strip_variant_patterns<T: PatternChar>(
    variant: &mut syn::Variant,
) -> syn::Result<Option<Pattern<T>>> {
    let mut attrs = variant.attrs.extract_if(.., |a| {
        let Some(ident) = a.meta.path().get_ident() else {
            return false;
        };

        ident == "pattern"
    });

    let Some(attr) = attrs.next() else {
        return Ok(None);
    };

    if let Some(a) = attrs.next() {
        return Err(syn::Error::new(a.span(), "Duplicated pattern attributes."));
    }

    let name_value = attr.meta.require_name_value()?;
    let pattern = Pattern::new(&name_value.value)?;
    Ok(Some(pattern))
}

fn strip_variant_declares<T: PatternChar>(variant: &mut syn::Variant) -> syn::Result<Declares<T>> {
    let attrs = variant.attrs.extract_if(.., |a| {
        let Some(ident) = a.meta.path().get_ident() else {
            return false;
        };

        ident == "declare"
    });

    let mut declares = Declares::new();
    for attr in attrs {
        let meta = attr.meta.require_list()?;
        let decl: Declares<T> = syn::parse2(meta.tokens.clone())?;
        declares.merge(decl);
    }

    Ok(declares)
}
