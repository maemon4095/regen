use syn::spanned::Spanned;

pub struct FieldAttribute {
    builder: Option<syn::Path>,
}

impl FieldAttribute {
    pub fn builder(&self) -> Option<&syn::Path> {
        self.builder.as_ref()
    }
}

pub fn strip_field_attribute(field: &mut syn::Field) -> syn::Result<FieldAttribute> {
    let attrs = field.attrs.extract_if(.., |e| {
        let Some(ident) = e.meta.path().get_ident() else {
            return false;
        };

        ident == "builder"
    });

    let mut builder = None;
    for attr in attrs {
        let ident = attr.meta.path().get_ident().unwrap();

        if ident == "builder" {
            let name_value = attr.meta.require_name_value()?;
            let path = match &name_value.value {
                syn::Expr::Path(p) => p,
                _ => {
                    return Err(syn::Error::new(
                        name_value.value.span(),
                        "builder path was expected.",
                    ));
                }
            };

            builder = Some(path.path.clone());
        }
    }

    Ok(FieldAttribute { builder })
}
