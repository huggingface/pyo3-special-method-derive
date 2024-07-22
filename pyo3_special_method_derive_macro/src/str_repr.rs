use crate::{
    ATTR_NAMESPACE, ATTR_NAMESPACE_FORMATTER, ATTR_NAMESPACE_NO_FMT_SKIP, ATTR_NAMESPACE_REPR,
    ATTR_NAMESPACE_STR,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Attribute, DeriveInput, Fields, Ident, LitStr, Token, Visibility};
macro_rules! create_body {
    ($input:expr, $ident:expr, $is_repr:expr) => {
        match &$input.data {
            syn::Data::Struct(s) => {
                generate_fmt_impl_for_struct(s, $ident, $is_repr, Some(&$input.attrs))
            }
            syn::Data::Enum(e) => {
                generate_fmt_impl_for_enum(e, $ident, $is_repr, Some(&$input.attrs))
            }
            syn::Data::Union(u) => {
                let error = syn::Error::new_spanned(u.union_token, "Unions are not supported");
                return proc_macro2::TokenStream::from(error.into_compile_error());
            }
        }
    };
}

const DEFAULT_ENUM_IDENT_FORMATTER: &str = "{}.{}";
const DEFAULT_ELEMENT_FORMATTER: &str = "{}";
const DEFAULT_STRUCT_IDENT_FORMATTER: &str = "{}({})";

pub(crate) enum DeriveType {
    ForAutoDisplay,
    ForAutoDebug,
}

// Internal function to generate impls of the custom trait: `ExtensionRepr|ExtensionStr{ident}`
pub(crate) fn impl_formatter(input: &DeriveInput, ty: DeriveType) -> proc_macro2::TokenStream {
    // Get the name of the struct
    let ident = &input.ident;
    // Determine if the implementation is for a "repr" type
    let is_repr = matches!(ty, DeriveType::ForAutoDebug);

    // Create body for display and debug
    let body_display = create_body!(input, ident, is_repr);
    let body_debug = create_body!(input, ident, is_repr);

    // Determine which traits to implement
    match ty {
        DeriveType::ForAutoDisplay => {
            quote! {
                impl pyo3_special_method_derive::PyDisplay for #ident {
                    fn fmt_display(&self) -> String {
                        use pyo3_special_method_derive::PyDisplay;
                        #body_display
                        repr
                    }
                }
            }
        }
        DeriveType::ForAutoDebug => {
            quote! {
                impl pyo3_special_method_derive::PyDebug for #ident {
                    fn fmt_debug(&self) -> String {
                        use pyo3_special_method_derive::PyDebug;
                        #body_debug
                        repr
                    }
                }
            }
        }
    }
}

fn generate_fmt_impl_for_struct(
    data_struct: &syn::DataStruct,
    name: &Ident,
    is_repr: bool,
    string_formatter: Option<&Vec<Attribute>>,
) -> proc_macro2::TokenStream {
    let mut ident_formatter = quote! { #DEFAULT_STRUCT_IDENT_FORMATTER };
    if let Some(attrs) = string_formatter {
        for attr in attrs {
            if attr.path().is_ident(ATTR_NAMESPACE_FORMATTER) {
                if let Some(formatter) = find_display_attribute(attr) {
                    ident_formatter = formatter;
                    break;
                }
                break;
            }
        }
    }

    let fields = &data_struct.fields;
    let fields = fields
        .iter()
        .filter(|f| {
            // Default `is_skip` based on the field's visibility
            let mut to_skip = !matches!(f.vis, Visibility::Public(_));
            let namespace = if is_repr {
                ATTR_NAMESPACE_REPR
            } else {
                ATTR_NAMESPACE_STR
            };

            for attr in &f.attrs {
                let path = attr.path();
                if path.is_ident(ATTR_NAMESPACE)
                    || path.is_ident(ATTR_NAMESPACE_FORMATTER)
                    || path.is_ident(namespace)
                {
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("skip") {
                            to_skip = true;
                        }
                        Ok(())
                    });
                    break;
                } else if path.is_ident(ATTR_NAMESPACE_NO_FMT_SKIP) {
                    // Explicitly mark to not skip the field
                    to_skip = false;
                    break;
                }
            }
            !to_skip
        })
        .collect::<Vec<_>>();
    let field_fmts = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let display_attr = {
                let mut display_attr = None;

                for attr in &field.attrs {
                    let path = attr.path();
                    if path.is_ident(ATTR_NAMESPACE_FORMATTER) {
                        display_attr = Some(attr);
                    }
                }

                display_attr
            };

            let mut variant_fmt = quote! { #DEFAULT_ELEMENT_FORMATTER };
            if let Some(display_attr) = display_attr {
                if let Some(formatter) = find_display_attribute(display_attr) {
                    variant_fmt = formatter;
                }
            }

            let formatters = variant_fmt.to_string().matches("{}").count()
                - variant_fmt.to_string().matches("{{}}").count();
            if formatters > 1 {
                panic!("Specify 1 (variant), or 0 formatters in the format string.");
            };

            let formatter_str = variant_fmt.to_string();

            let format_str = format!("{{}}={}{{}}", &formatter_str[1..formatter_str.len()-1]);

            let postfix = if i + 1 < fields.len() { ", " } else { "" };
            let formatter = if is_repr { quote! { fmt_debug } } else { quote! { fmt_display } };
            match &field.ident {
                Some(ident) => {
                    quote! {
                        repr += &format!(#format_str, stringify!(#ident), self.#ident.#formatter(), #postfix);
                    }
                }
                None => {
                    // If the field doesn't have a name, we generate a name based on its index
                    let index = syn::Index::from(i);
                    quote! {
                        repr += &format!(#format_str, stringify!(#index), self.#index.#formatter(), #postfix);
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    // Handle any escaped {}
    let formatters = ident_formatter.to_string().matches("{}").count()
        - ident_formatter.to_string().matches("{{}}").count();
    let ident_formatter = if formatters == 2 {
        quote! { format!(#ident_formatter, stringify!(#name), repr) }
    } else if formatters == 1 {
        quote! { format!(#ident_formatter, stringify!(#name)) }
    } else if formatters == 0 {
        quote! { format!(#ident_formatter) }
    } else {
        panic!("Specify 2 (name, repr), 1 (name), or 0 formatters in the format string.")
    };

    quote! {
        let mut repr = "".to_string();
        #(#field_fmts)*

        let repr = #ident_formatter;
    }
}

// Define a struct to hold the parsed tokens
struct FmtAttribute {
    ident: Ident,
    _eq_token: Token![=],
    pub lit_str: LitStr,
}

// Implement parsing for the FmtAttribute struct
impl Parse for FmtAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        let _eq_token: Token![=] = input.parse()?;
        let lit_str: LitStr = input.parse()?;
        Ok(FmtAttribute {
            ident,
            _eq_token,
            lit_str,
        })
    }
}

pub fn find_display_attribute(attr: &Attribute) -> Option<TokenStream> {
    // Parse the attribute arguments
    let attribute = match attr.parse_args::<FmtAttribute>() {
        Ok(display_macro) => Some(display_macro),
        Err(e) => {
            e.to_compile_error();
            None
        }
    };

    // Check if we have a valid attribute and return the literal as TokenStream
    if let Some(attr) = attribute {
        if attr.ident == "fmt" {
            let list_str = attr.lit_str;
            Some(quote! { #list_str })
        } else {
            None
        }
    } else {
        None
    }
}

fn generate_fmt_impl_for_enum(
    data_enum: &syn::DataEnum,
    name: &Ident,
    is_repr: bool,
    string_formatter: Option<&Vec<Attribute>>,
) -> proc_macro2::TokenStream {
    let variants = data_enum.variants.iter().collect::<Vec<_>>();
    let mut ident_formatter = quote! { #DEFAULT_ENUM_IDENT_FORMATTER };
    if let Some(attrs) = string_formatter {
        for attr in attrs {
            if attr.path().is_ident(ATTR_NAMESPACE_FORMATTER) {
                if let Some(formatter) = find_display_attribute(attr) {
                    ident_formatter = formatter;
                    break;
                }
                break;
            }
        }
    }

    let arms = variants.iter().map(|variant| {
        let ident = &variant.ident;
        let (to_skip, display_attr) = {
            let mut to_skip = false;
            let mut display_attr = None;
            let namespace = if is_repr {
                ATTR_NAMESPACE_REPR
            } else {
                ATTR_NAMESPACE_STR
            };

            for attr in &variant.attrs {
                let path = attr.path();
                if path.is_ident(ATTR_NAMESPACE) || path.is_ident(ATTR_NAMESPACE_FORMATTER) || path.is_ident(namespace) {
                    let _ = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("skip") {
                            to_skip = true;
                        }
                        Ok(())
                    });
                    if path.is_ident(ATTR_NAMESPACE_FORMATTER) {
                        display_attr = Some(attr);
                    }
                }
            }

            (to_skip, display_attr)
        };

        let mut variant_fmt = quote! { #DEFAULT_ELEMENT_FORMATTER };
        if let Some(display_attr) = display_attr {
            if let Some(formatter) = find_display_attribute(display_attr) {
                variant_fmt = formatter;
            }
        }

        // If {} is not in ident_fmt, we must not format ident.
        // If {} is not in variant_fmt, we don't use stringify! either
        match &variant.fields {
            Fields::Unit => {
                let formatters = variant_fmt.to_string().matches("{}").count()
                    - variant_fmt.to_string().matches("{{}}").count();
                let variant_formatter = if formatters == 1 {
                    quote! { &format!(#variant_fmt, stringify!(#ident)) }
                } else if formatters == 0 {
                    quote! { &format!(#variant_fmt) }
                } else {
                    panic!("Specify 1 (variant), or 0 formatters in the format string.")
                };

                if !to_skip {
                    quote! {
                        Self::#ident => repr += #variant_formatter,
                    }
                } else {
                    quote! {
                        Self::#ident => repr += "<variant skipped>",
                    }
                }
            }
            syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                // Tuple variant with one field
                if !to_skip {
                    quote! { #name::#ident(ref single) => {#ident_formatter;} }
                } else {
                    quote! {
                        #ident => repr += "<variant skipped>",
                    }
                }  // TODO now that we have AutoDisplay we want this
            }
            Fields::Named(fields) => {
                let formatters = variant_fmt.to_string().matches("{}").count()
                    - variant_fmt.to_string().matches("{{}}").count();
                if formatters > 1 {
                    panic!("Specify 1 (variant), or 0 formatters in the format string.");
                };

                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let mut format_string = "{}(".to_string();
                let formatter = if is_repr { quote! { fmt_debug } } else { quote! { fmt_display } };
                for (i, name) in field_names.iter().enumerate() {
                    let formatter_str = variant_fmt.to_string();
                    if i == 0 {
                        format_string = format!("{format_string}{}={}", name.as_ref().unwrap(), &formatter_str[1..formatter_str.len()-1]);
                    } else {
                        format_string = format!("{format_string}, {}={}", name.as_ref().unwrap(), &formatter_str[1..formatter_str.len()-1]);
                    }
                }
                format_string = format!("{format_string})");
                if !to_skip {
                    let mut names = Vec::new();
                    for name in field_names.clone() {
                        names.push(quote! { #name.#formatter() });
                    }
                    if formatters > 0 {
                        quote! {
                            Self::#ident { #(#field_names),* } => repr += &format!(#format_string, stringify!(#ident), #(#names),*),
                        }
                    } else {
                        quote! {
                            Self::#ident { #(#field_names),* } => repr += &format!(#format_string, stringify!(#ident)),
                        }
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
            _ => {
                // Default case: stringify the variant name
                quote! {  &format!("{}", stringify!(#ident)); }
            }
        }
    }).collect::<Vec<_>>();

    // Handle any escaped {}
    let formatters = ident_formatter.to_string().matches("{}").count()
        - ident_formatter.to_string().matches("{{}}").count();
    let ident_formatter = if formatters == 2 {
        quote! { format!(#ident_formatter, stringify!(#name), repr) }
    } else if formatters == 1 {
        quote! { format!(#ident_formatter, stringify!(#name)) }
    } else if formatters == 0 {
        quote! { format!(#ident_formatter) }
    } else {
        panic!("Specify 2 (name, repr), 1 (name), or 0 formatters in the format string.")
    };

    quote! {
        let mut repr = "".to_string();
        match self {
            #(#arms)*
        }
        let repr = #ident_formatter;
    }
}
