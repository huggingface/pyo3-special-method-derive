//! Derive macros to help with Python

extern crate proc_macro;
use dir::get_dir_enum_variants;
use proc_macro::TokenStream;
use quote::quote;
use str_repr::display_debug_derive;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

mod dir;
mod str_repr;

// TODO: We should only list fields which are at least readable by Python users.
// This would require either reading the visibility modifier (easier) or checking
// the `pyo3` getter.

// TODO: We should have a skip attribute macro, similar to how serde has attribute
// macros on fields?

/// Add a `__dir__` method to the struct in a `#[pymethods]` impl
#[proc_macro_derive(DirHelper, attributes(skip))]
pub fn dir_helper_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let name = &input.ident;

    // Generate code to match the struct's fields
    let expanded = match input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    // If the struct has named fields extract their names
                    let field_names = fields
                        .named
                        .iter()
                        .filter(|f| !f.attrs.iter().any(|attr| attr.path().is_ident("skip")))
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();

                    if field_names.is_empty() {
                        quote! {
                            #[pyo3::pymethods]
                            impl #name {
                                pub fn __dir__(&self) -> Vec<String> {
                                    Vec::new()
                                }
                            }
                        }
                    } else {
                        // Prepare an array where the elements are expressions that prepare the field vec
                        let mut assigner = proc_macro2::TokenStream::new();
                        quote_into::quote_into!(assigner += [#{
                            for name in field_names {
                                quote_into::quote_into!(assigner += (names.push(stringify!(#name).to_string())),)
                            }
                        }];);
                        quote! {
                            #[pyo3::pymethods]
                            impl #name {
                                pub fn __dir__(&self) -> Vec<String> {
                                    let mut names = Vec::new();
                                    #assigner
                                    names
                                }
                            }
                        }
                    }
                }
                Fields::Unit => {
                    // If the struct has no fields
                    quote! {
                        #[pyo3::pymethods]
                        impl #name {
                            pub fn __dir__(&self) -> Vec<String> {
                                Vec::new()
                            }
                        }
                    }
                }
                Fields::Unnamed(_) => {
                    quote! {
                        compile_error!("Unnamed fields for struct are not supported for DirHelper derive.");
                    }
                }
            }
        }
        Data::Enum(e) => {
            let variants = get_dir_enum_variants(&e);
            let mut assigner = proc_macro2::TokenStream::new();
            quote_into::quote_into!(assigner += [#{
                for name in variants {
                    quote_into::quote_into!(assigner += (names.push(stringify!(#name).to_string())),)
                }
            }];);
            quote! {
                #[pyo3::pymethods]
                impl #name {
                    pub fn __dir__(&self) -> Vec<String> {
                        let mut names = Vec::new();
                        #assigner
                        names
                    }
                }
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Unions are not supported for DirHelper derive");
            }
        }
    };
    TokenStream::from(expanded)
}

/// Add `__str__` and `__repr__` methods to the struct in a `#[pymethods]` impl
#[proc_macro_derive(StrReprHelper, attributes(skip))]
pub fn str_repr_helper_derive(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);

    // Get the name of the struct
    let name = &input.ident;

    let display_debug_derive_body = display_debug_derive(&input);

    let expanded = quote! {
        #display_debug_derive_body

        #[pyo3::pymethods]
        impl #name {
            pub fn __str__(&self) -> String {
                format!("{self}")
            }

            pub fn __repr__(&self) -> String {
                format!("{self:?}")
            }
        }
    };

    TokenStream::from(expanded)
}
