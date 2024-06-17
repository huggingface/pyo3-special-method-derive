//! Derive macros to help with Rust PyO3 support.
//!
//! This crate automatically derives the following functions for structs and enums:
//! - `__str__`
//! - `__repr__`
//! - `__dir__`
//! - `__getattr__`
//! - `__dict__`
//!
//! - Skip exposure of variants or fields with the `#[attr]` attribute
//! - Skip variants or fields for `__str__` or `__repr__` differently with the `#[pyo3_smd_str(skip)]` and `#[pyo3_smd_repr(skip)]` attributes
//! - Struct fields which are not `pub` are skipped automatically
//!

extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use str_repr::{impl_formatter, StrOrRepr};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Visibility};

mod str_repr;

const ATTR_NAMESPACE: &str = "pyo3_smd";
const ATTR_NAMESPACE_STR: &str = "pyo3_smd_str";
const ATTR_NAMESPACE_REPR: &str = "pyo3_smd_repr";

/// Add a `__dir__` method to a struct or enum.
///
/// - Skip exposure of certain fields by adding the `#[pyo3_smd(skip)]` attribute macro
/// - For structs, all fields are skipped which are not marked `pub`
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Dir;
/// #[pyclass]
/// #[derive(Dir)]
/// struct Person {
///     pub name: String,
///     address: String,
///     #[pyo3_smd(skip)]
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(Dir, attributes(pyo3_smd))]
pub fn dir_derive(input: TokenStream) -> TokenStream {
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
                        compile_error!("Unnamed fields for struct are not supported for Dir derive.");
                    }
                }
            }
        }
        Data::Enum(e) => {
            let matchers = e.variants.iter()
            .filter(|variant| {
                !variant.attrs.iter().any(|attr| {
                    let mut is_skip = false;
                    attr.parse_nested_meta(|meta| {
                        is_skip = meta.path.is_ident("skip");
                        Ok(())
                    })
                    .unwrap();
                    attr.path().is_ident(ATTR_NAMESPACE) && is_skip
                })
            })
                .map(|variant| {
                    let ident = &variant.ident;
                    match &variant.fields {
                        Fields::Unit => {
                            quote! {
                                Self::#ident => { vec![] }
                            }
                        }
                        Fields::Unnamed(_) => {
                            unreachable!("Unnamed fields are not supported for enums with PyO3.")
                        }
                        Fields::Named(fields) => {
                            let field_names = fields.named.iter().map(|f| f.ident.as_ref().unwrap().clone()).collect::<Vec<_>>();

                            let mut assigner = proc_macro2::TokenStream::new();
                            quote_into::quote_into!(assigner += [#{
                                for name in &field_names {
                                    quote_into::quote_into!(assigner += (names.push(stringify!(#name).to_string())),)
                                }
                            }];);

                            quote! {
                                Self::#ident { .. } => {
                                    let mut names = Vec::new();
                                    #assigner
                                    names
                                }
                            }
                        }
                    }
                });
            let skipped_matchers = e
                .variants
                .iter()
                .filter(|variant| {
                    variant.attrs.iter().any(|attr| {
                        let mut is_skip = false;
                        attr.parse_nested_meta(|meta| {
                            is_skip = meta.path.is_ident("skip");
                            Ok(())
                        })
                        .unwrap();
                        attr.path().is_ident(ATTR_NAMESPACE) && is_skip
                    })
                })
                .map(|variant| {
                    let ident = &variant.ident;
                    match &variant.fields {
                        Fields::Unit => {
                            quote! {
                                Self::#ident => { vec![] }
                            }
                        }
                        Fields::Unnamed(_) => {
                            unreachable!("Unnamed fields are not supported for enums with PyO3.")
                        }
                        Fields::Named(_) => {
                            quote! {
                                Self::#ident { .. } => { vec![] }
                            }
                        }
                    }
                });
            quote! {
                #[pyo3::pymethods]
                impl #name {
                    pub fn __dir__(&self) -> Vec<String> {
                        match self {
                            #(#matchers)*
                            #(#skipped_matchers)*
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Unions are not supported for Dir derive");
            }
        }
    };
    TokenStream::from(expanded)
}

/// Add a `__str__` method to the struct or enum.
///
/// - Skip printing of certain fields by adding the `#[pyo3_smd(skip)]` attribute macro
/// - To specialze skipping for `__str__`, use the `#[pyo3_smd_str(skip)]` attributes
/// - For structs, all fields are skipped which are not marked `pub`
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Str;
/// #[pyclass]
/// #[derive(Str)]
/// struct Person {
///     pub name: String,
///     address: String,
///     #[pyo3_smd(skip)]
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(Str, attributes(pyo3_smd, pyo3_smd_str))]
pub fn str_derive(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);

    // Get the name of the struct
    let name = &input.ident;

    let display_debug_derive_body = impl_formatter(&input, StrOrRepr::ForStr);

    let expanded = quote! {
        #display_debug_derive_body

        #[pyo3::pymethods]
        impl #name {
            pub fn __str__(&self) -> String {
                let mut output = String::new();
                self.str_fmt(&mut output);
                output
            }
        }
    };

    TokenStream::from(expanded)
}

/// Add a `__repr__` method to the struct or enum.
///
/// - Skip printing of certain fields by adding the `#[pyo3_smd(skip)]` attribute macro
/// - To specialze skipping for `__repr__`, use the `#[pyo3_smd_repr(skip)]` attributes
/// - For structs, all fields are skipped which are not marked `pub`
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Repr;
/// #[pyclass]
/// #[derive(Repr)]
/// struct Person {
///     pub name: String,
///     address: String,
///     #[pyo3_smd(skip)]
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(Repr, attributes(pyo3_smd, pyo3_smd_repr))]
pub fn repr_derive(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);

    // Get the name of the struct
    let name = &input.ident;

    let display_debug_derive_body = impl_formatter(&input, StrOrRepr::ForRepr);

    let expanded = quote! {
        #display_debug_derive_body

        #[pyo3::pymethods]
        impl #name {
            pub fn __repr__(&self) -> String {
                let mut output = String::new();
                self.repr_fmt(&mut output);
                output
            }
        }
    };

    TokenStream::from(expanded)
}

/// Add a `__getattr__` method to a struct or enum.
///
/// - For structs, all fields are skipped which are not marked `pub`
/// - Skip printing of certain fields or variants by adding the `#[pyo3_smd(skip)]` attribute macro
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Getattr;
/// #[pyclass]
/// #[derive(Getattr)]
/// struct Person {
///     pub name: String,
///     address: String,
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(Getattr, attributes(pyo3_smd))]
pub fn getattr_derive(input: TokenStream) -> TokenStream {
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
                                    Err(pyo3::exceptions::PyAttributeError::new_err(format!("'{}' has no attribute '{attr}'", stringify!(#name))))
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
                        compile_error!("Unnamed fields for struct are not supported for Getattr derive.");
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            let variants = data_enum.variants.iter().collect::<Vec<_>>();
            let match_arms = variants.iter()
            .filter(|variant| {
                !variant.attrs.iter().any(|attr| {
                    let mut is_skip = false;
                    attr.parse_nested_meta(|meta| {
                        is_skip = meta.path.is_ident("skip");
                        Ok(())
                    })
                    .unwrap();
                    attr.path().is_ident(ATTR_NAMESPACE) && is_skip
                })
            })
                .map(|variant| {
                let ident = &variant.ident;
                match &variant.fields {
                    Fields::Unit => {
                        quote! {
                            Self::#ident => Err(pyo3::exceptions::PyAttributeError::new_err(format!("'{}.{}' has no attribute '{attr}'", stringify!(#name), stringify!(#ident)))),
                        }
                    }
                    Fields::Unnamed(_) => {
                        unreachable!("Unnamed fields are not supported for enums with PyO3.")
                    }
                    Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| f.ident.as_ref().unwrap().clone()).collect::<Vec<_>>();
                        let mut inserter = Vec::new();
                        for ident_name in &field_names {
                            inserter.push(
                                quote! {
                                    stringify!(#ident_name) => {
                                        Ok(pyo3::Python::with_gil(|py| #ident_name.clone().into_py(py)))
                                    }
                                }
                            );
                        }
                        inserter.push(
                            quote! {
                                _ => Err(pyo3::exceptions::PyAttributeError::new_err(format!("'{}.{}' has no attribute '{attr}'", stringify!(#name), stringify!(#ident))))
                            }
                        );
                        quote! {
                            Self::#ident { #(#field_names),* } => {
                                match attr.as_str() {
                                    #(#inserter)*
                                }
                            }
                        }
                    }
                }
            }).collect::<Vec<_>>();
            let ignored_match_arms = variants.iter()
            .filter(|variant| {
                variant.attrs.iter().any(|attr| {
                    let mut is_skip = false;
                    attr.parse_nested_meta(|meta| {
                        is_skip = meta.path.is_ident("skip");
                        Ok(())
                    })
                    .unwrap();
                    attr.path().is_ident(ATTR_NAMESPACE) && is_skip
                })
            })
                .map(|variant| {
                let ident = &variant.ident;
                // If a variant was ignored always raise an exception
                match &variant.fields {
                    Fields::Unit => {
                        quote! {
                            Self::#ident => Err(pyo3::exceptions::PyAttributeError::new_err(format!("'{}.{}' has no attribute '{attr}'", stringify!(#name), stringify!(#ident)))),
                        }
                    }
                    Fields::Unnamed(_) => {
                        unreachable!("Unnamed fields are not supported for enums with PyO3.")
                    }
                    Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| f.ident.as_ref().unwrap().clone()).collect::<Vec<_>>();
                        quote! {
                            Self::#ident { #(#field_names),* } => {
                                let _ = (#(#field_names),*);
                                Err(pyo3::exceptions::PyAttributeError::new_err(format!("'{}.{}' has no attribute '{attr}'", stringify!(#name), stringify!(#ident))))
                            }
                        }
                    }
                }
            }).collect::<Vec<_>>();
            quote! {
                #[pyo3::pymethods]
                impl #name {
                    #[allow(non_snake_case)]
                    pub fn __getattr__(&self, attr: String) -> pyo3::PyResult<pyo3::Py<pyo3::PyAny>> {
                        use pyo3::IntoPy;

                        match self {
                            #(#match_arms)*
                            #(#ignored_match_arms)*
                        }
                    }
                }
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Unions are not supported for Getattr derive");
            }
        }
    };
    expanded.into()
}

/// Add a `__dict__` attribute to a struct or enum.
///
/// - For structs, all fields are skipped which are not marked `pub`
/// - Skip printing of certain fields or variants by adding the `#[pyo3_smd(skip)]` attribute macro
///
/// ## Example
/// ```
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Dict;
/// #[pyclass]
/// #[derive(Dict)]
/// struct Person {
///     pub name: String,
///     address: String,
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(Dict, attributes(pyo3_smd))]
pub fn dict_derive(input: TokenStream) -> TokenStream {
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
                        .filter(|f| matches!(f.vis, Visibility::Public(_)))
                        .map(|f| f.ident.as_ref().unwrap())
                        .collect::<Vec<_>>();

                    if field_names.is_empty() {
                        quote! {
                            #[pyo3::pymethods]
                            impl #name {
                                #[allow(non_snake_case)]
                                #[getter]
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
                                #[getter]
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
                            #[getter]
                            pub fn __dict__(&self) -> std::collections::HashMap<String, pyo3::Py<pyo3::PyAny>> {
                                std::collections::HashMap::new()
                            }
                        }
                    }
                }
                Fields::Unnamed(_) => {
                    quote! {
                        compile_error!("Unnamed fields for struct are not supported for Dict derive.");
                    }
                }
            }
        }
        Data::Enum(data_enum) => {
            let variants = data_enum.variants.iter().collect::<Vec<_>>();
            let match_arms = variants.iter()
            .filter(|variant| {
                !variant.attrs.iter().any(|attr| {
                    let mut is_skip = false;
                    attr.parse_nested_meta(|meta| {
                        is_skip = meta.path.is_ident("skip");
                        Ok(())
                    })
                    .unwrap();
                    attr.path().is_ident(ATTR_NAMESPACE) && is_skip
                })
            })
                .map(|variant| {
                let ident = &variant.ident;
                match &variant.fields {
                    Fields::Unit => {
                        quote! {
                            Self::#ident => { }
                        }
                    }
                    Fields::Unnamed(_) => {
                        unreachable!("Unnamed fields are not supported for enums with PyO3.")
                    }
                    Fields::Named(fields) => {
                        let field_names = fields.named.iter().map(|f| f.ident.as_ref().unwrap().clone()).collect::<Vec<_>>();
                        let mut inserter = Vec::new();
                        for name in &field_names {
                            inserter.push(
                                quote! {
                                    values.insert(
                                            stringify!(#name).to_string(), pyo3::Python::with_gil(|py| #name.clone().into_py(py))
                                    );
                                }
                            );
                        }
                        quote! {
                            Self::#ident { #(#field_names),* } => {
                                #(#inserter)*
                            }
                        }
                    }
                }
            }).collect::<Vec<_>>();
            let ignored_match_arms = variants
                .iter()
                .filter(|variant| {
                    variant.attrs.iter().any(|attr| {
                        let mut is_skip = false;
                        attr.parse_nested_meta(|meta| {
                            is_skip = meta.path.is_ident("skip");
                            Ok(())
                        })
                        .unwrap();
                        attr.path().is_ident(ATTR_NAMESPACE) && is_skip
                    })
                })
                .map(|variant| {
                    let ident = &variant.ident;
                    // If a variant was ignored just output no __dict__ data.
                    match &variant.fields {
                        Fields::Unit => {
                            quote! {
                                Self::#ident => { }
                            }
                        }
                        Fields::Unnamed(_) => {
                            unreachable!("Unnamed fields are not supported for enums with PyO3.")
                        }
                        Fields::Named(fields) => {
                            let field_names = fields
                                .named
                                .iter()
                                .map(|f| f.ident.as_ref().unwrap().clone())
                                .collect::<Vec<_>>();

                            quote! {
                                Self::#ident { #(#field_names),* } => {
                                    let _ = (#(#field_names),*);
                                }
                            }
                        }
                    }
                })
                .collect::<Vec<_>>();
            quote! {
                #[pyo3::pymethods]
                impl #name {
                    #[allow(non_snake_case)]
                    #[getter]
                    pub fn __dict__(&self) -> std::collections::HashMap<String, pyo3::Py<pyo3::PyAny>> {
                        use pyo3::IntoPy;

                        let mut values = std::collections::HashMap::new();
                        match self {
                            #(#match_arms)*
                            #(#ignored_match_arms)*
                        }
                        values
                    }
                }
            }
        }
        Data::Union(_) => {
            quote! {
                compile_error!("Unions are not supported for Dict derive");
            }
        }
    };
    expanded.into()
}
