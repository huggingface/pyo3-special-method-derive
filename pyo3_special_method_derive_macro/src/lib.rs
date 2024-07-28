extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use str_repr::{impl_formatter, DeriveType};
use syn::{parse_macro_input, Data, DeriveInput, Fields, Visibility};

mod str_repr;

const ATTR_SKIP_NAMESPACE: &str = "skip";
const ATTR_NAMESPACE_NO_FMT_SKIP: &str = "pyo3_fmt_no_skip";
const ATTR_NAMESPACE_FORMATTER: &str = "format";
const SKIP_ALL: &str = "All";

/// Add a `__dir__` method to a struct or enum.
///
/// - Skip exposure of certain fields by adding `Dir` to the `#[skip(...)]` attribute macro: `#[skip(Dir)]`
/// - For structs, all fields are skipped which are not marked `pub`
/// - Skip exposure of certain fields for all derive macros by adding `All`: `#[skip(All)]`
///
/// ## Example
/// ```ignore
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
#[proc_macro_derive(Dir, attributes(skip))]
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
                                if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                                    // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                                    attr.parse_nested_meta(|meta| {
                                        is_skip |= meta.path.is_ident("Dir")
                                            || meta.path.is_ident(SKIP_ALL);
                                        Ok(())
                                    })
                                    .unwrap();
                                }
                                is_skip
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
                        if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                            // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                            attr.parse_nested_meta(|meta| {
                                is_skip |= meta.path.is_ident("Dir") || meta.path.is_ident(SKIP_ALL);
                                Ok(())
                            })
                            .unwrap();
                        }
                        is_skip
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

                            if field_names.is_empty() {
                                quote! { Self::#ident { .. } => { vec![] } }
                            } else {
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
                    }
                });
            let skipped_matchers = e
                .variants
                .iter()
                .filter(|variant| {
                    variant.attrs.iter().any(|attr| {
                        let mut is_skip = false;
                        if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                            // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                            attr.parse_nested_meta(|meta| {
                                is_skip |=
                                    meta.path.is_ident("Dir") || meta.path.is_ident(SKIP_ALL);
                                Ok(())
                            })
                            .unwrap();
                        };
                        is_skip
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
/// This expects every type for which its field or variant is not skipped to implement the PyDisplay trait.
/// Certain implementations are automatically provided, but you can implement the required trait yourself
/// or use a provided convenience macro.
///
/// - Skip exposure of certain fields by adding `Str` to the `#[skip(...)]` attribute macro: `#[skip(Str)]`
/// - For structs, all fields are skipped which are not marked `pub`
/// - Skip exposure of certain fields for all derive macros by adding `All`: `#[skip(All)]`
///
/// The `formatter` attribute macro, when used to annotate an enum, controls how the type name and variant are formatted.
/// By default it is `{}.{}`. The format string takes 2 (filled in as name, variant), 1 (filled in as name), or 0 formatters:
///
/// ```ignore
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Str;
/// #[pyclass]
/// #[derive(Str)]
/// #[format(fmt = "{}.{}")]
/// enum Person {
///     Alive,
///     Dead
/// }
/// ```
///
/// The `formatter` attribute macro, when used to annotate an enum, controls how the type name and fields are formatted.
/// By default it is `{}({})`. The format string takes 2 (filled in as name, fields), 1 (filled in as name), or 0 formatters:
///
/// ```ignore
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Str;
/// #[pyclass]
/// #[derive(Str)]
/// #[format(fmt = "{}({})")]
/// struct Mountain {
///     pub height: usize,
/// }
/// ```
///
/// - A struct field may be annotated with `#[format(fmt = ...)]` where the format string can take 1 (field) or 0 formatters.
/// - An enum variant may be annotated with `#[format(fmt = ...)]` where the format string can take 1 (variant) or 0 formatters.
///
/// ## Example
/// ```ignore
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
#[proc_macro_derive(Str, attributes(skip, pyo3_fmt_no_skip, format))]
pub fn str_derive(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);

    // Get the name of the struct
    let name = &input.ident;

    let display_derive_body = match impl_formatter(&input, DeriveType::ForAutoDisplay, "Str") {
        Ok(x) => x,
        Err(e) => e.into_compile_error(),
    };

    let expanded = quote! {
        #display_derive_body

        #[pyo3::pymethods]
        impl #name {
            pub fn __str__(&self) -> String {
                use pyo3_special_method_derive::PyDisplay;
                self.fmt_display()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implement `PyDisplay` on a struct or enum. Implements only `PyDisplay`, no `Display`.
///
/// This has the same requirements and behavior of [`Str`].
///
/// The `formatter` attribute macro, when used to annotate an enum, controls how the type name and variant are formatted.
/// By default it is `{}.{}`. The format string takes 2 (filled in as name, variant), 1 (filled in as name), or 0 formatters:
///
/// ```ignore
/// use pyo3_special_method_derive::AutoDisplay;
/// #[derive(AutoDisplay)]
/// #[format(fmt = "{}.{}")]
/// enum Person {
///     Alive,
///     Dead
/// }
/// ```
///
/// The `formatter` attribute macro, when used to annotate an enum, controls how the type name and fields are formatted.
/// By default it is `{}({})`. The format string takes 2 (filled in as name, fields), 1 (filled in as name), or 0 formatters:
///
/// ```ignore
/// use pyo3_special_method_derive::AutoDisplay;
/// #[derive(AutoDisplay)]
/// #[format(fmt = "{}({})")]
/// struct Mountain {
///     pub height: usize,
/// }
/// ```
///
/// - A struct field may be annotated with `#[format(fmt = ...)]` where the format string can take 1 (field) or 0 formatters.
/// - An enum variant may be annotated with `#[format(fmt = ...)]` where the format string can take 1 (variant) or 0 formatters.
///
/// ## Example
///
/// The `formatter` also has other uses, outlined below:
///
/// ```ignore
/// use pyo3_special_method_derive::AutoDisplayOnly;
/// #[derive(AutoDisplayOnly)]
/// struct Person {
///     pub name: String,
///     address: String,
///     #[format(skip)]
///     pub phone_number: String,
///     #[format] // -> force display of private field
///     hash: u32,
/// }
/// ```
#[proc_macro_derive(AutoDisplayOnly, attributes(skip, pyo3_fmt_no_skip, format))]
pub fn auto_display_only(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);

    let display_debug_derive_body =
        impl_formatter(&input, DeriveType::ForAutoDisplay, "AutoDisplay");

    let display_debug_derive_body = match display_debug_derive_body {
        Ok(x) => x,
        Err(e) => e.into_compile_error(),
    };

    TokenStream::from(display_debug_derive_body)
}

/// Implement `PyDisplay` on a struct or enum. Implements `Display` based on `PyDisplay`.
///
/// See the docs of [`AutoDisplayOnly`].
#[proc_macro_derive(AutoDisplay, attributes(skip, pyo3_fmt_no_skip, format))]
pub fn auto_display(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);
    let name = &input.ident;

    let display_debug_derive_body =
        impl_formatter(&input, DeriveType::ForAutoDisplay, "AutoDisplay");

    let display_debug_derive_body = match display_debug_derive_body {
        Ok(x) => x,
        Err(e) => e.into_compile_error(),
    };

    let expanded = quote! {
        #display_debug_derive_body

        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use pyo3_special_method_derive::PyDisplay;
                write!(f, "{}", self.fmt_display())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Add a `__repr__` method to the struct or enum.
///
/// This expects every type for which its field or variant is not skipped to implement the PyDebug trait.
/// Certain implementations are automatically provided, but you can implement the required trait yourself
/// or use a provided convenience macro.
///
/// - Skip exposure of certain fields by adding `Repr` to the `#[skip(...)]` attribute macro: `#[skip(Repr)]`
/// - For structs, all fields are skipped which are not marked `pub`
/// - Skip exposure of certain fields for all derive macros by adding `All`: `#[skip(All)]`
///
/// The `formatter` attribute macro, when used to annotate an enum, controls how the type name and variant are formatted.
/// By default it is `{}.{}`. The format string takes 2 (filled in as name, variant), 1 (filled in as name), or 0 formatters:
///
/// ```ignore
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Repr;
/// #[pyclass]
/// #[format(fmt = "{}.{}")]
/// enum Person {
///     Alive,
///     Dead
/// }
/// ```
///
/// The `formatter` attribute macro, when used to annotate an enum, controls how the type name and fields are formatted.
/// By default it is `{}({})`. The format string takes 2 (filled in as name, fields), 1 (filled in as name), or 0 formatters:
///
/// ```ignore
/// use pyo3::pyclass;
/// use pyo3_special_method_derive::Repr;
/// #[pyclass]
/// #[format(fmt = "{}({})")]
/// struct Mountain {
///     pub height: usize,
/// }
/// ```
///
/// - A struct field may be annotated with `#[format(fmt = ...)]` where the format string can take 1 (field) or 0 formatters.
/// - An enum variant may be annotated with `#[format(fmt = ...)]` where the format string can take 1 (variant) or 0 formatters.
///
/// ## Example
/// ```ignore
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
#[proc_macro_derive(Repr, attributes(skip, pyo3_fmt_no_skip, format))]
pub fn repr_derive(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);

    // Get the name of the struct
    let name = &input.ident;

    let display_debug_derive_body = impl_formatter(&input, DeriveType::ForAutoDebug, "Repr");

    let display_debug_derive_body = match display_debug_derive_body {
        Ok(x) => x,
        Err(e) => e.into_compile_error(),
    };

    let expanded = quote! {
        #display_debug_derive_body

        #[pyo3::pymethods]
        impl #name {
            pub fn __repr__(&self) -> String {
                use pyo3_special_method_derive::PyDebug;
                self.fmt_debug()
            }
        }
    };

    TokenStream::from(expanded)
}

/// Implement `PyDebug` on a struct or enum. Implements only `PyDebug`, no `Debug`.
///
/// This has the same requirements and behavior of [`Repr`].
///
/// The `formatter` attribute macro, when used to annotate an enum, controls how the type name is formatted.
/// By default it is `{}.{}`. The format string takes 2 (filled in as name, variant), 1 (filled in as name), or 0 formatters:
///
/// ```ignore
/// use pyo3_special_method_derive::AutoDebug;
/// #[derive(AutoDebug)]
/// #[format(fmt = "{}.{}")]
/// enum Person {
///     Alive,
///     Dead
/// }
/// ```
///
/// The `formatter` attribute macro, when used to annotate an enum, controls how the type name and fields are formatted.
/// By default it is `{}({})`. The format string takes 2 (filled in as name, fields), 1 (filled in as name), or 0 formatters:
///
/// ```ignore
/// use pyo3_special_method_derive::AutoDebug;
/// #[derive(AutoDebug)]
/// #[format(fmt = "{}({})")]
/// struct Mountain {
///     pub height: usize,
/// }
/// ```
///
/// - A struct field may be annotated with `#[format(fmt = ...)]` where the format string can take 1 (field) or 0 formatters.
/// - An enum variant may be annotated with `#[format(fmt = ...)]` where the format string can take 1 (variant) or 0 formatters.
///
/// ## Example
///
/// The `auto_debug` also has other uses, outlined below:
///
/// ## Example
/// ```ignore
/// use pyo3_special_method_derive::AutoDebug;
/// #[derive(AutoDebug)]
/// struct Person {
///     pub name: String,
///     address: String,
///     #[pyo3_smd(skip)]
///     pub phone_number: String,
/// }
/// ```
#[proc_macro_derive(AutoDebugOnly, attributes(skip, pyo3_fmt_no_skip, format))]
pub fn auto_debug_only(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);

    let display_debug_derive_body = impl_formatter(&input, DeriveType::ForAutoDebug, "AutoDebug");

    let display_debug_derive_body = match display_debug_derive_body {
        Ok(x) => x,
        Err(e) => e.into_compile_error(),
    };

    TokenStream::from(display_debug_derive_body)
}

/// Implement `PyDebug` on a struct or enum. Implements `Debug` based on `PyDebug`.
///
/// See the docs of [`AutoDisplayOnly`].
#[proc_macro_derive(AutoDebug, attributes(skip, pyo3_fmt_no_skip, format))]
pub fn auto_debug(input_stream: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input_stream as DeriveInput);

    let display_debug_derive_body = impl_formatter(&input, DeriveType::ForAutoDebug, "AutoDebug");

    let display_debug_derive_body = match display_debug_derive_body {
        Ok(x) => x,
        Err(e) => e.into_compile_error(),
    };

    let name = &input.ident;
    let expanded = quote! {
        #display_debug_derive_body

        impl std::fmt::Debug for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                use pyo3_special_method_derive::PyDebug;
                write!(f, "{}", self.fmt_debug())
            }
        }
    };

    TokenStream::from(expanded)
}

/// Add a `__getattr__` method to a struct or enum.
///
/// - For structs, all fields are skipped which are not marked `pub`
/// - Skip exposure of certain fields by adding `Getattr` to the `#[skip(...)]` attribute macro: `#[skip(Getattr)]`
/// - Skip exposure of certain fields for all derive macros by adding `All`: `#[skip(All)]`
///
/// ## Example
/// ```ignore
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
#[proc_macro_derive(Getattr, attributes(skip))]
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
                                if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                                    // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                                    attr.parse_nested_meta(|meta| {
                                        is_skip |= meta.path.is_ident("Getattr")
                                            || meta.path.is_ident(SKIP_ALL);
                                        Ok(())
                                    })
                                    .unwrap();
                                };
                                is_skip
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
                    if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                        // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                        attr.parse_nested_meta(|meta| {
                            is_skip |= meta.path.is_ident("Getattr") || meta.path.is_ident(SKIP_ALL);
                            Ok(())
                        })
                        .unwrap();
                    };
                    is_skip
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
                    if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                        // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                        attr.parse_nested_meta(|meta| {
                            is_skip |= meta.path.is_ident("Getattr") || meta.path.is_ident(SKIP_ALL);
                            Ok(())
                        })
                        .unwrap();
                    };
                    is_skip
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
/// - Skip exposure of certain fields by adding `Dict` to the `#[skip(...)]` attribute macro: `#[skip(Dict)]`
/// - Skip exposure of certain fields for all derive macros by adding `All`: `#[skip(All)]`
///
/// ## Example
/// ```ignore
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
#[proc_macro_derive(Dict, attributes(skip))]
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
                                if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                                    // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                                    attr.parse_nested_meta(|meta| {
                                        is_skip |= meta.path.is_ident("Dict")
                                            || meta.path.is_ident(SKIP_ALL);
                                        Ok(())
                                    })
                                    .unwrap();
                                };
                                is_skip
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
                    if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                        // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                        attr.parse_nested_meta(|meta| {
                            is_skip |= meta.path.is_ident("Dict") || meta.path.is_ident(SKIP_ALL);
                            Ok(())
                        })
                        .unwrap();
                    };
                    is_skip
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
                        if attr.path().is_ident(ATTR_SKIP_NAMESPACE) {
                            // only parse ATTR_SKIP_NAMESPACE and not [serde] or [default]
                            attr.parse_nested_meta(|meta| {
                                is_skip |=
                                    meta.path.is_ident("Dict") || meta.path.is_ident(SKIP_ALL);
                                Ok(())
                            })
                            .unwrap();
                        };
                        is_skip
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
