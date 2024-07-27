use crate::{
    ATTR_NAMESPACE, ATTR_NAMESPACE_FORMATTER, ATTR_NAMESPACE_NO_FMT_SKIP, ATTR_NAMESPACE_REPR,
    ATTR_NAMESPACE_STR,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Error, Fields, Ident, LitStr, Token, Visibility};
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
                return Ok(proc_macro2::TokenStream::from(error.into_compile_error()));
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
pub(crate) fn impl_formatter(
    input: &DeriveInput,
    ty: DeriveType,
) -> syn::Result<proc_macro2::TokenStream> {
    // Get the name of the struct
    let ident = &input.ident;
    // Determine if the implementation is for a "repr" type
    let is_repr = matches!(ty, DeriveType::ForAutoDebug);

    // Create body for display and debug
    let body_display = create_body!(input, ident, is_repr)?;
    let body_debug = create_body!(input, ident, is_repr)?;

    // Determine which traits to implement
    match ty {
        DeriveType::ForAutoDisplay => Ok(quote! {
            impl pyo3_special_method_derive_0_21::PyDisplay for #ident {
                fn fmt_display(&self) -> String {
                    use pyo3_special_method_derive_0_21::PyDisplay;
                    #body_display
                    repr
                }
            }
        }),
        DeriveType::ForAutoDebug => Ok(quote! {
            impl pyo3_special_method_derive_0_21::PyDebug for #ident {
                fn fmt_debug(&self) -> String {
                    use pyo3_special_method_derive_0_21::PyDebug;
                    #body_debug
                    repr
                }
            }
        }),
    }
}

fn generate_fmt_impl_for_struct(
    data_struct: &syn::DataStruct,
    name: &Ident,
    is_repr: bool,
    string_formatter: Option<&Vec<Attribute>>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut ident_formatter = quote! { #DEFAULT_STRUCT_IDENT_FORMATTER };
    if let Some(attrs) = string_formatter {
        for attr in attrs {
            if attr.path().is_ident(ATTR_NAMESPACE_FORMATTER) {
                match find_display_attribute(attr){
                    Ok(Some(formatter)) => ident_formatter = formatter,
                    Err(error) => return Err(error),
                    Ok(None)=>{},
                }
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
                    let res = attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("skip") {
                            to_skip = true;
                        }
                        Ok(())
                    });
                    if res.is_err() && path.is_ident(ATTR_NAMESPACE_FORMATTER) {
                        to_skip = false;
                    }
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
                match find_display_attribute(display_attr) {
                    Ok(Some(formatter)) => variant_fmt = formatter,
                    Ok(None)=>{},
                    Err(e) => return Err(e), 
                }
            }

            let formatters = variant_fmt.to_string().matches("{}").count()
                + variant_fmt.to_string().matches("{{}}").count();
            if formatters > 1 {
                return Err(syn::Error::new(data_struct.struct_token.span(), "Specify 1 (variant), or 0 formatters in the format string."));
            };
            let formatter_str = variant_fmt.to_string();
            let format_str = format!("{{}}={}{{}}", &formatter_str[1..formatter_str.len()-1]);

            let postfix = if i + 1 < fields.len() { ", " } else { "" };
            let formatter = if is_repr { quote! { fmt_debug } } else { quote! { fmt_display } };
            Ok(match &field.ident {
                            Some(ident) => {
                                if formatters > 0 {
                                    quote! {
                                        repr += &format!(#format_str, stringify!(#ident), self.#ident.#formatter(), #postfix);
                                    }
                                } else {
                                    quote! {
                                        repr += &format!(#format_str, stringify!(#ident), #postfix);
                                    }
                                }
                            }
                            None => {
                                // If the field doesn't have a name, we generate a name based on its index
                                let index = syn::Index::from(i);
                                print!("{:?}:{:?}, Number of formatters: {}", index.clone(), postfix,formatters.clone());

                                if formatters == 1 {
                                    quote! {
                                        repr += &format!(#format_str, stringify!(#index), self.#index.#formatter(), #postfix);
                                    }
                                } else {
                                    quote! {
                                        repr += &format!(#format_str, stringify!(#index), #postfix);
                                    }
                                }
                            }
                        })
        })
        .collect::<syn::Result<Vec<_>>>()?;

    // Handle any escaped {}
    let formatters = ident_formatter.to_string().matches("{}").count()
        + ident_formatter.to_string().matches("{{}}").count();
    let ident_formatter = if formatters == 2 {
        quote! { format!(#ident_formatter, stringify!(#name), repr) }
    } else if formatters == 1 {
        quote! { format!(#ident_formatter, repr) }
    } else if formatters == 0 {
        quote! { format!(#ident_formatter) }
    } else {
        return Err(syn::Error::new(
            data_struct.struct_token.span(),
            "Specify 2 (name, repr), 1 (name), or 0 formatters in the format string.",
        ));
    };

    Ok(quote! {
        let mut repr = "".to_string();
        #(#field_fmts)*

        let repr = #ident_formatter;
    })
}

// Define a struct to hold the parsed tokens
struct FmtAttribute {
    ident: Option<Ident>,
    _eq_token: Option<Token![=]>,
    pub lit_str: Option<LitStr>,
}

// Implement parsing for the FmtAttribute struct
impl Parse for FmtAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = match input.parse(){
            Ok(ident) => ident,
            Err(_) => None // skip #[format]
        };
        let _eq_token = match input.parse(){
            Ok(token) => token,
            Err(_) => None, // skip #[format(skip)]
        };
        let lit_str = match input.parse(){
            Ok(str_format) => str_format,
            Err(_) => None, // skip #[format(skip)]
        }; // if we have xxx = , then we parse
        Ok(FmtAttribute {
            ident,
            _eq_token,
            lit_str,
        })
    }
}

// Parse the provided attribute. Returns the appropriate erro if it fails.
pub fn find_display_attribute(attr: &Attribute) -> Result<Option<TokenStream>, Error> {
    // Parse the attribute arguments
    let attribute = attr.parse_args::<FmtAttribute>();
    match attribute {
        Ok(fmt_attr) => 
            match fmt_attr.ident {
                Some(fmt_ident) =>
                    {
                        if fmt_ident == "skip" {
                            return Ok(None);
                        } else if fmt_ident == "fmt"{
                            if let Some(list_str) = fmt_attr.lit_str{
                                return Ok(Some(quote! { #list_str }));
                            }
                        }
                        return Err(syn::Error::new_spanned(attr, "Error parsing fmt, ident wrong or not lit str"));
                    } 
                _ => return Ok(None),
            }
        Err(error) =>{println!("You used #[format]: {:?}", attr); Ok(None)},
    }
}

fn generate_fmt_impl_for_enum(
    data_enum: &syn::DataEnum,
    name: &Ident,
    is_repr: bool,
    string_formatter: Option<&Vec<Attribute>>,
) -> syn::Result<proc_macro2::TokenStream> {
    let variants = data_enum.variants.iter().collect::<Vec<_>>();
    let formatter = if is_repr {
        quote! { fmt_debug }
    } else {
        quote! { fmt_display }
    };
    let mut ident_formatter = quote! { #DEFAULT_ENUM_IDENT_FORMATTER };
    if let Some(attrs) = string_formatter {
        for attr in attrs {
            if attr.path().is_ident(ATTR_NAMESPACE_FORMATTER) {
                match find_display_attribute(attr){
                    Ok(Some(formatter)) => ident_formatter = formatter,
                    Ok(None)=>{},
                    Err(error) => return Err(error),   
                }
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
            match find_display_attribute(display_attr){
                Ok(Some(fmt)) => variant_fmt = fmt,
                Err(error) => return Err(error),
                Ok(None)=>{},
            };
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
                    return Err(syn::Error::new(data_enum.enum_token.span(), "Specify 1 (variant), or 0 formatters in the format string."));
                };

                Ok(if !to_skip {
                    quote! {
                        Self::#ident => repr += #variant_formatter,
                    }
                } else {
                    quote! {
                        Self::#ident => repr += "<variant skipped>",
                    }
                })
            }
            syn::Fields::Unnamed(fields) => {
                // Tuple variant with one field
                // TODO now that we have AutoDisplay we want this
                let mut enum_representation = TokenStream::new();
                println!("{:?}", &variant.ident);
                let field_repr = if !to_skip {
                    let field_value = &variant.ident;
                    quote! { #name::#field_value(single) => {repr += &format!("{}", single.#formatter());}, }
                } else {
                    quote! {
                        #ident => repr += "<variant skipped>",
                    }
                };
                enum_representation = quote!{#enum_representation #field_repr};
               Ok(enum_representation)
            }
            Fields::Named(fields) => {
                let mut field_names: Vec<(&Option<Ident>, String, usize)> = Vec::new();
                for field in &fields.named {
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
                        match find_display_attribute(display_attr) {
                            Ok(Some(formatter)) => variant_fmt = formatter,
                            Err(error) => return Err(error),
                            Ok(None)=>{},
                        }
                    }

                    let formatters = variant_fmt.to_string().matches("{}").count()
                        - variant_fmt.to_string().matches("{{}}").count();
                    if formatters > 1 {
                        return Err(syn::Error::new(data_enum.enum_token.span(), "Specify 1 (variant), or 0 formatters in the format string."));
                    };
                    let formatter_str = variant_fmt.to_string();

                    field_names.push((&field.ident, formatter_str[1..formatter_str.len()-1].to_string(), formatters));
                }

                let mut format_string = "{}(".to_string();
                for (i, (name, formatter, _n_formatters)) in field_names.iter().enumerate() {
                    if i == 0 {
                        format_string = format!("{format_string}{}={}", name.as_ref().unwrap(), formatter);
                    } else {
                        format_string = format!("{format_string}, {}={}", name.as_ref().unwrap(), formatter);
                    }
                }
                format_string = format!("{format_string})");
                Ok(if !to_skip {
                    let mut names = Vec::new();
                    for (name, _, n_formatters) in field_names.clone() {
                        if n_formatters > 0 {
                            names.push(quote! { #name.#formatter() });
                        }
                    }
                    let mut new_field_names = Vec::new();
                    for (name, _, _) in field_names.clone() {
                        new_field_names.push(name);
                    }
                    quote! {
                        Self::#ident { #(#new_field_names),* } => repr += &format!(#format_string, stringify!(#ident), #(#names),*),
                    }
                } else {
                    let mut names = Vec::new();
                    for (name, _, _) in field_names.clone() {
                        names.push(quote! { #name });
                    }
                    quote! {
                        Self::#ident { #(#names),* } => {
                            let _ = (#(#names),*);
                            repr += "<variant skipped>";
                        }
                    }
                })
            }
            _ => {
                // Default case: stringify the variant name
                Ok(quote! {  &format!("{}", stringify!(#ident)); })
            }
        }
    }).collect::<syn::Result<Vec<_>>>()?;

    // Handle any escaped {}
    let formatters = ident_formatter.to_string().matches("{}").count()
        + ident_formatter.to_string().matches("{{}}").count();
    let ident_formatter = if formatters == 2 {
        quote! { format!(#ident_formatter, stringify!(#name), repr) }
    } else if formatters == 1 {
        quote! { format!(#ident_formatter, repr) }
    } else if formatters == 0 {
        quote! { format!(#ident_formatter) }
    } else {
        return Err(syn::Error::new(
            data_enum.enum_token.span(),
            "Specify 2 (name, repr), 1 (name), or 0 formatters in the format string.",
        ));
    };

    Ok(quote! {
        let mut repr = "".to_string();
        match self {
            #(#arms)*
        }
        let repr = #ident_formatter;
    })
}
