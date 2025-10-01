use crate::{declares::Declares, expr::PatternChar};

pub fn strip_prelude<T: PatternChar>(
    item: &mut syn::ItemEnum,
) -> Result<RegenPrelude<T>, syn::Error> {
    let declare_attrs = item.attrs.extract_if(.., |a| {
        let Some(ident) = a.meta.path().get_ident() else {
            return false;
        };

        ident == "declare"
    });

    let mut declares = Declares::new();

    for attr in declare_attrs {
        let list = attr.meta.require_list()?;
        let decl: Declares<T> = syn::parse2(list.tokens.clone())?;
        declares.merge(decl);
    }

    Ok(RegenPrelude { declares })
}

pub struct RegenPrelude<T: PatternChar> {
    pub declares: Declares<T>,
}
