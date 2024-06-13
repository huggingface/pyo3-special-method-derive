use quote::quote;
use syn::{DeriveInput, Fields, Ident};

macro_rules! create_body {
    ($input:expr, $ident:expr) => {
        match &$input.data {
            syn::Data::Struct(s) => generate_fmt_impl_for_struct(s),
            syn::Data::Enum(e) => generate_fmt_impl_for_enum(e, $ident),
            syn::Data::Union(u) => {
                let error = syn::Error::new_spanned(u.union_token, "Unions are not supported");
                return proc_macro2::TokenStream::from(error.into_compile_error());
            }
        }
    };
}

// Internal function to generate Display and Debug impls.
// `Display` is used for `__str__`. `Debug` is used for `__repr__`.
pub(crate) fn display_debug_derive(input: &DeriveInput) -> proc_macro2::TokenStream {
    // Get the name of the struct
    let ident = &input.ident;

    let body_display = create_body!(input, ident);

    let body_debug = create_body!(input, ident);

    if matches!(input.data, syn::Data::Struct(_)) {
        quote! {
            impl std::fmt::Debug for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}(", stringify!(#ident))?;
                    #(#body_debug)*
                    write!(f, ")")
                }
            }

            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    write!(f, "{}(", stringify!(#ident))?;
                    #(#body_display)*
                    write!(f, ")")
                }
            }
        }
    } else {
        quote! {
            impl std::fmt::Debug for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    match self {
                        #(#body_debug)*
                    }
                    write!(f, "")
                }
            }

            impl std::fmt::Display for #ident {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    match self {
                        #(#body_display)*
                    }
                    write!(f, "")
                }
            }
        }
    }
}

fn generate_fmt_impl_for_struct(data_struct: &syn::DataStruct) -> Vec<proc_macro2::TokenStream> {
    let fields = &data_struct.fields;
    let fields = fields
        .iter()
        .filter(|f| !f.attrs.iter().any(|attr| attr.path().is_ident("skip")))
        .collect::<Vec<_>>();
    let field_fmts = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            // TODO: handle if debug. This may be checking visibility or only
            // displaying user specified idents. For now, repr the fields in Debug.
            let postfix = if i + 1 < fields.len() { ", " } else { "" };
            match &field.ident {
                Some(ident) => {
                    quote! {
                        write!(f, "{}={:?}{}", stringify!(#ident), self.#ident, #postfix)?;
                    }
                }
                None => {
                    // If the field doesn't have a name, we generate a name based on its index
                    let index = syn::Index::from(i);
                    quote! { write!(f, "{}={:?}{}", stringify!(#index), self.#index, #postfix)?; }
                }
            }
        })
        .collect::<Vec<_>>();
    // Collect the mapped tokens into a TokenStream
    field_fmts
}

fn generate_fmt_impl_for_enum(
    data_enum: &syn::DataEnum,
    name: &Ident,
) -> Vec<proc_macro2::TokenStream> {
    let variants = data_enum.variants.iter().collect::<Vec<_>>();
    variants.iter()
        .map(|variant| {
        let ident = &variant.ident;
        let to_skip = variant.attrs.iter().any(|attr| attr.path().is_ident("skip"));
        match &variant.fields {
            Fields::Unit => {
                if !to_skip {
                    quote! {
                        Self::#ident => write!(f, "{}.{}", stringify!(#name), stringify!(#ident))?,
                    }
                } else {
                    quote! {
                        Self::#ident => write!(f, "<variant skipped>")?,
                    }
                }
            }
            Fields::Unnamed(_) => {
                unreachable!("Unnamed fields are not supported for enums with PyO3.")
            }
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let mut format_string = "{}.{}(".to_string();
                for (i, name) in field_names.iter().enumerate() {
                    if i == 0 {
                        format_string = format!("{format_string}{}={{}}", name.as_ref().unwrap());
                    } else {
                        format_string = format!("{format_string}, {}={{}}", name.as_ref().unwrap());
                    }
                }
                format_string = format!("{format_string})");
                if !to_skip {
                    quote! {
                        Self::#ident { #(#field_names),* } => write!(f, #format_string, stringify!(#name), stringify!(#ident), #(#field_names),*)?,
                    }
                } else {
                    quote! {
                        Self::#ident { #(#field_names),* } => {
                            let _ = (#(#field_names),*);
                            write!(f, "<variant skipped>")?
                        }
                    }
                }
            }
        }
    }).collect::<Vec<_>>()
}
