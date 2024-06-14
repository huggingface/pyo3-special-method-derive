//! Derive macros to help with Rust PyO3 support.
//!
//! This crate automatically derives the following functions for structs and enums:
//! - `__str__`
//! - `__repr__`
//! - `__dir__`
//!
//! Note: When using the `StrReprHelper` macro. if `T` did not use `StrReprHelper`, it requires `T: Debug` for each `T` inside the item. The `Debug` trait is used for the outputs.
//!
//! - You can skip exposure of variants or fields with the `#[attr]` attribute
//! - You can skip variants or fields for `__str__` or `__repr__` differently with the `#[skip_str]` and `#[skip_repr]` attributes
//! - Struct fields which are not `pub` are skipped automatically
//!

extern crate proc_macro;
use dir::get_dir_enum_variants;
use proc_macro::TokenStream;
use quote::quote;
use str_repr::display_debug_derive;
use syn::{parse_macro_input, Data, DeriveInput, Fields, Visibility};

mod dir;
mod str_repr;

/// Add a `__dir__` method to the struct in a `#[pymethods]` impl.
///
/// - You can skip exposure of certain fields by adding the `#[skip]` attribute macro
/// - For structs, all fields are skipped which are not marked `pub`
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::DirHelper;
/// #[pyclass]
/// #[derive(DirHelper)]
/// struct Person {
///     pub name: String,
///     address: String,
///     #[skip]
///     pub phone_number: String,
/// }
/// ```
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
                        .filter(|f| matches!(f.vis, Visibility::Public(_)))
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

/// Add `__str__` and `__repr__` methods to the struct in a `#[pymethods]` impl.
///
/// - You can skip printing of certain fields by adding the `#[skip]` attribute macro
/// - To specialze skipping depending on `__str__` and `__repr__`, you can use the `#[skip_str]`
/// and `#[skip_repr]` attributes which skip for `__str__` and `__repr__` respectively
/// - For structs, all fields are skipped which are not marked `pub`
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::StrReprHelper;
/// #[pyclass]
/// #[derive(StrReprHelper)]
/// struct Person {
///     pub name: String,
///     address: String,
///     #[skip]
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(StrReprHelper, attributes(skip, skip_str, skip_repr))]
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

/// Add `__getattr__` to a struct in a `#[pymethods]` impl.
///
/// - For structs, all fields are skipped which are not marked `pub`
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::GetattrHelper;
/// #[pyclass]
/// #[derive(GetattrHelper)]
/// struct Person {
///     pub name: String,
///     address: String,
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(GetattrHelper)]
pub fn getattr_helper_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = match input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    // If the struct has named fields extract their names
                    let field_names = fields
                        .named
                        .iter()
                        .filter(|f| matches!(f.vis, Visibility::Public(_)))
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();
                    let field_names_str = field_names
                        .iter()
                        .map(|f| f.to_string())
                        .collect::<Vec<_>>();

                    if field_names.is_empty() {
                        quote! {
                            #[pyo3::pymethods]
                            impl #name {
                                #[allow(non_snake_case)]
                                pub fn __getattr__(&self, attr: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
                                    Err(pyo3::exceptions::PyAttributeError::new_err(format!("'{}' has no attribute '{attr}'", #name)))
                                }
                            }
                        }
                    } else {
                        // Prepare an array where the elements are expressions that prepare the field vec
                        let mut matchers = Vec::new();
                        for (name, ident) in field_names_str.iter().zip(field_names) {
                            let inner = quote! {
                                #name => {
                                    Ok(pyo3::Python::with_gil(|py| self.#ident.clone().into_py(py)))
                                }
                            };
                            matchers.push(inner);
                        }

                        quote! {
                            #[pyo3::pymethods]
                            impl #name {
                                #[allow(non_snake_case)]
                                pub fn __getattr__(&self, attr: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
                                    use pyo3::IntoPy;

                                    match attr.as_str() {
                                        #(#matchers)*
                                        name => Err(pyo3::exceptions::PyAttributeError::new_err(format!("'{}' has no attribute '{attr}'", stringify!(#name))))
                                    }
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
                            #[allow(non_snake_case)]
                            pub fn __getattr__(&self, attr: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
                                Err(pyo3::exceptions::PyAttributeError::new_err(format!("'{}' has no attribute '{attr}'", #name)))
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
        Data::Enum(_) => {
            quote! {
                compile_error!("Enums are not supported for GetattrHelper derive");
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Unions are not supported for GetattrHelper derive");
            }
        }
    };
    expanded.into()
}

/// Add `__getattr__` to a struct in a `#[pymethods]` impl.
///
/// - For structs, all fields are skipped which are not marked `pub`
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::GetattrHelper;
/// #[pyclass]
/// #[derive(GetattrHelper)]
/// struct Person {
///     pub name: String,
///     address: String,
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(DictHelper)]
pub fn dict_helper_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let expanded = match input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    // If the struct has named fields extract their names
                    let field_names = fields
                        .named
                        .iter()
                        .filter(|f| matches!(f.vis, Visibility::Public(_)))
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();

                    if field_names.is_empty() {
                        quote! {
                            #[pyo3::pymethods]
                            impl #name {
                                #[allow(non_snake_case)]
                                pub fn __dict__(&self) -> std::collections::HashMap<String, pyo3::Py<pyo3::PyAny>> {
                                    std::collections::HashMap::new()
                                }
                            }
                        }
                    } else {
                        // Prepare an array where the elements are expressions that prepare the field vec
                        let mut inserter = Vec::new();
                        for name in field_names {
                            inserter.push(
                                quote! {
                                    values.insert(
                                            stringify!(#name).to_string(), pyo3::Python::with_gil(|py| self.#name.clone().into_py(py))
                                    );
                                }
                            );
                        }

                        quote! {
                            #[pyo3::pymethods]
                            impl #name {
                                #[allow(non_snake_case)]
                                pub fn __dict__(&self) -> std::collections::HashMap<String, pyo3::Py<pyo3::PyAny>> {
                                    use pyo3::IntoPy;

                                    let mut values = std::collections::HashMap::new();
                                    #(#inserter)*
                                    values
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
                            #[allow(non_snake_case)]
                            pub fn __dict__(&self) -> std::collections::HashMap<String, pyo3::Py<pyo3::PyAny>> {
                                std::collections::HashMap::new()
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
        Data::Enum(_) => {
            quote! {
                compile_error!("Enums are not supported for GetattrHelper derive");
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Unions are not supported for GetattrHelper derive");
            }
        }
    };
    expanded.into()
}
