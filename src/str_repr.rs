use proc_macro2::Span;
use quote::quote;
use syn::{DeriveInput, Fields, Ident, Visibility};

macro_rules! create_body {
    ($input:expr, $ident:expr, $is_repr:expr) => {
        match &$input.data {
            syn::Data::Struct(s) => generate_fmt_impl_for_struct(s, $is_repr),
            syn::Data::Enum(e) => generate_fmt_impl_for_enum(e, $ident, $is_repr),
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

    let body_display = create_body!(input, ident, false);

    let body_debug = create_body!(input, ident, true);

    if matches!(input.data, syn::Data::Struct(_)) {
        let trait_name = Ident::new(
            &format!("ExtensionStrRepr{}", ident.to_string()),
            Span::call_site(),
        );
        quote! {
            trait #trait_name {
                fn repr_fmt(&self, f: &mut String);
                fn str_fmt(&self, f: &mut String);
            }
            impl #trait_name for #ident {
                fn repr_fmt(&self, f: &mut String) {
                    *f += &format!("{}(", stringify!(#ident));
                    #(#body_debug)*
                    *f += ")";
                }
                fn str_fmt(&self, f: &mut String) {
                    *f += &format!("{}(", stringify!(#ident));
                    #(#body_display)*
                    *f += ")";
                }
            }
        }
    } else {
        let trait_name = Ident::new(
            &format!("ExtensionStrRepr{}", ident.to_string()),
            Span::call_site(),
        );
        quote! {
            trait #trait_name {
                fn repr_fmt(&self, f: &mut String);
                fn str_fmt(&self, f: &mut String);
            }
            impl #trait_name for #ident {
                fn repr_fmt(&self, f: &mut String) {
                    match self {
                        #(#body_debug)*
                    }
                }
                fn str_fmt(&self, f: &mut String) {
                    match self {
                        #(#body_display)*
                    }
                }
            }
        }
    }
}

fn generate_fmt_impl_for_struct(
    data_struct: &syn::DataStruct,
    is_repr: bool,
) -> Vec<proc_macro2::TokenStream> {
    let fields = &data_struct.fields;
    let fields = fields
        .iter()
        .filter(|f| !f.attrs.iter().any(|attr| attr.path().is_ident("skip")))
        .filter(|f| {
            if is_repr {
                !f.attrs.iter().any(|attr| attr.path().is_ident("skip_repr"))
            } else {
                !f.attrs.iter().any(|attr| attr.path().is_ident("skip_str"))
            }
        })
        .filter(|f| matches!(f.vis, Visibility::Public(_)))
        .collect::<Vec<_>>();
    let field_fmts = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let postfix = if i + 1 < fields.len() { ", " } else { "" };
            match &field.ident {
                Some(ident) => {
                    quote! {
                        *f += &format!("{}={:?}{}", stringify!(#ident), self.#ident, #postfix);
                    }
                }
                None => {
                    // If the field doesn't have a name, we generate a name based on its index
                    let index = syn::Index::from(i);
                    quote! { *f += &format!("{}={:?}{}", stringify!(#index), self.#index, #postfix); }
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
    is_repr: bool,
) -> Vec<proc_macro2::TokenStream> {
    let variants = data_enum.variants.iter().collect::<Vec<_>>();
    variants.iter()
        .map(|variant| {
        let ident = &variant.ident;
        let to_skip = variant.attrs.iter().any(|attr| attr.path().is_ident("skip") || if is_repr {
            attr.path().is_ident("skip_repr")
        } else {
            attr.path().is_ident("skip_str")
        });
        match &variant.fields {
            Fields::Unit => {
                if !to_skip {
                    quote! {
                        Self::#ident => *f += &format!("{}.{}", stringify!(#name), stringify!(#ident)),
                    }
                } else {
                    quote! {
                        Self::#ident => *f += "<variant skipped>",
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
                        format_string = format!("{format_string}{}={{:?}}", name.as_ref().unwrap());
                    } else {
                        format_string = format!("{format_string}, {}={{:?}}", name.as_ref().unwrap());
                    }
                }
                format_string = format!("{format_string})");
                if !to_skip {
                    quote! {
                        Self::#ident { #(#field_names),* } => *f += &format!(#format_string, stringify!(#name), stringify!(#ident), #(#field_names),*),
                    }
                } else {
                    quote! {
                        Self::#ident { #(#field_names),* } => {
                            let _ = (#(#field_names),*);
                            *f += "<variant skipped>";
                        }
                    }
                }
            }
        }
    }).collect::<Vec<_>>()
}
