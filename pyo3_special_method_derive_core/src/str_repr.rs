use quote::quote;
use syn::{DeriveInput, Fields, Ident, Visibility};

use crate::{ATTR_NAMESPACE, ATTR_NAMESPACE_REPR, ATTR_NAMESPACE_STR};

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

pub(crate) enum StrOrRepr {
    ForStr,
    ForRepr,
}

// Internal function to generate impls of the custom trait: `ExtensionRepr|ExtensionStr{ident}`
pub(crate) fn impl_formatter(input: &DeriveInput, ty: StrOrRepr) -> proc_macro2::TokenStream {
    // Get the name of the struct
    let ident = &input.ident;

    let body_display = create_body!(input, ident, matches!(ty, StrOrRepr::ForRepr));

    let body_debug = create_body!(input, ident, matches!(ty, StrOrRepr::ForRepr));

    if matches!(input.data, syn::Data::Struct(_)) {
        match ty {
            StrOrRepr::ForStr => {
                quote! {
                    impl pyo3_special_method_derive_lib::PyDisplay for #ident {
                        fn fmt_display(&self) -> String {
                            // TODO
                            use pyo3_special_method_derive_lib::PyDebug;
                            use pyo3_special_method_derive_lib::PyDisplay;

                            let mut repr = "".to_string();
                            repr += &format!("{}(", stringify!(#ident));
                            #(#body_display)*
                            repr += ")";
                            repr
                        }
                    }
                }
            }
            StrOrRepr::ForRepr => {
                quote! {
                    impl pyo3_special_method_derive_lib::PyDebug for #ident {
                        fn fmt_debug(&self) -> String {
                            use pyo3_special_method_derive_lib::PyDebug;

                            let mut repr = "".to_string();
                            repr += &format!("{}(", stringify!(#ident));
                            #(#body_debug)*
                            repr += ")";
                            repr
                        }
                    }
                }
            }
        }
    } else {
        match ty {
            StrOrRepr::ForStr => {
                quote! {
                    impl pyo3_special_method_derive_lib::PyDisplay for #ident {
                        fn fmt_display(&self) -> String {
                            // TODO
                            use pyo3_special_method_derive_lib::PyDebug;
                            use pyo3_special_method_derive_lib::PyDisplay;

                            let mut repr = "".to_string();
                            match self {
                                #(#body_display)*
                            }
                            repr
                        }
                    }
                }
            }
            StrOrRepr::ForRepr => {
                quote! {
                    impl pyo3_special_method_derive_lib::PyDebug for #ident {
                        fn fmt_debug(&self) -> String {
                            use pyo3_special_method_derive_lib::PyDebug;

                            let mut repr = "".to_string();
                            match self {
                                #(#body_debug)*
                            }
                            repr
                        }
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
        .filter(|f| {
            !f.attrs.iter().any(|attr| {
                let mut is_skip = false;
                attr.parse_nested_meta(|meta| {
                    is_skip = meta.path.is_ident("skip");
                    Ok(())
                })
                .unwrap();
                attr.path().is_ident(ATTR_NAMESPACE) && is_skip
            })
        })
        .filter(|f| {
            let namespace = if is_repr {
                ATTR_NAMESPACE_REPR
            } else {
                ATTR_NAMESPACE_STR
            };
            !f.attrs.iter().any(|attr| {
                let mut is_skip = false;
                attr.parse_nested_meta(|meta| {
                    is_skip = meta.path.is_ident("skip");
                    Ok(())
                })
                .unwrap();
                attr.path().is_ident(namespace) && is_skip
            })
        })
        .filter(|f| matches!(f.vis, Visibility::Public(_)))
        .collect::<Vec<_>>();
    let field_fmts = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let postfix = if i + 1 < fields.len() { ", " } else { "" };
            let formatter = if is_repr { quote! { fmt_debug } } else { quote! { fmt_display } };
            match &field.ident {
                Some(ident) => {
                    quote! {
                        repr += &format!("{}={}{}", stringify!(#ident), self.#ident.#formatter(), #postfix);
                    }
                }
                None => {
                    // If the field doesn't have a name, we generate a name based on its index
                    let index = syn::Index::from(i);
                    quote! {
                        repr += &format!("{}={}{}", stringify!(#index), self.#index.#formatter(), #postfix);
                    }
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
            let to_skip = variant.attrs.iter().any(|attr| {
                let mut is_skip = false;
                attr.parse_nested_meta(|meta| {
                    is_skip = meta.path.is_ident("skip");
                    Ok(())
                })
                .unwrap();
                let skip_direct = attr.path().is_ident(ATTR_NAMESPACE) && is_skip;

                let namespace = if is_repr {
                    ATTR_NAMESPACE_REPR
                } else {
                    ATTR_NAMESPACE_STR
                };
                let mut is_skip = false;
                attr.parse_nested_meta(|meta| {
                    is_skip = meta.path.is_ident("skip");
                    Ok(())
                })
                .unwrap();
                let is_specific_skip = attr.path().is_ident(namespace) && is_skip;

                is_specific_skip || skip_direct
            }
            );
            match &variant.fields {
                Fields::Unit => {
                    if !to_skip {
                        quote! {
                            Self::#ident => repr += &format!("{}.{}", stringify!(#name), stringify!(#ident)),
                        }
                    } else {
                        quote! {
                            Self::#ident => repr += "<variant skipped>",
                        }
                    }
                }
                Fields::Unnamed(_) => {
                    unreachable!("Unnamed fields are not supported for enums with PyO3.")
                }
                Fields::Named(fields) => {
                    let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                    let mut format_string = "{}.{}(".to_string();
                    let formatter = if is_repr { quote! { fmt_debug } } else { quote! { fmt_display } };
                    for (i, name) in field_names.iter().enumerate() {
                        if i == 0 {
                            format_string = format!("{format_string}{}={{}}", name.as_ref().unwrap());
                        } else {
                            format_string = format!("{format_string}, {}={{}}", name.as_ref().unwrap());
                        }
                    }
                    format_string = format!("{format_string})");
                    if !to_skip {
                        let mut names = Vec::new();
                        for name in field_names.clone() {
                            names.push(quote! { #name.#formatter() });
                        }
                        quote! {
                            Self::#ident { #(#field_names),* } => repr += &format!(#format_string, stringify!(#name), stringify!(#ident), #(#names),*),
                        }
                    } else {
                        quote! {
                            Self::#ident { #(#field_names),* } => {
                                let _ = (#(#field_names),*);
                                repr += "<variant skipped>";
                            }
                        }
                    }
                }
            }
    }).collect::<Vec<_>>()
}
