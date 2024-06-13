use syn::Ident;

pub(crate) fn get_dir_enum_variants(data_enum: &syn::DataEnum) -> Vec<&Ident> {
    data_enum
        .variants
        .iter()
        .filter(|v| !v.attrs.iter().any(|attr| attr.path().is_ident("skip")))
        .map(|variant| &variant.ident)
        .collect::<Vec<_>>()
}
