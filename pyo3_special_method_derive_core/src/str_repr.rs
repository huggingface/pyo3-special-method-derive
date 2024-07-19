use crate::{ATTR_NAMESPACE, ATTR_NAMESPACE_NO_SKIP, ATTR_NAMESPACE_REPR, ATTR_NAMESPACE_STR};
use quote::quote;
use syn::MetaList;
use syn::{DeriveInput, Fields, Ident, Lit, Meta, MetaNameValue, Visibility, Attribute, Token, LitStr};
use syn::parse::{Parse, ParseStream};
use proc_macro2::TokenStream;
macro_rules! create_body {
    ($input:expr, $ident:expr, $is_repr:expr) => {
        match &$input.data {
            syn::Data::Struct(s) => generate_fmt_impl_for_struct(s, $ident, $is_repr),
            syn::Data::Enum(e) => generate_fmt_impl_for_enum(e, $ident, $is_repr, Some(&$input.attrs)),
            syn::Data::Union(u) => {
                let error = syn::Error::new_spanned(u.union_token, "Unions are not supported");
                return proc_macro2::TokenStream::from(error.into_compile_error());
            }
        }
    };
}

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
    let body_display = create_body!(input, ident, !is_repr);
    let body_debug = create_body!(input, ident, is_repr);


    // Determine which traits to implement
    let (ty_trait, ty_fn) = match ty {
        DeriveType::ForAutoDisplay => (
            quote! { impl std::fmt::Display },
            quote! {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {#body_display; write!(f, "{}", repr)}
            },
        ),
        DeriveType::ForAutoDebug => (
            quote! { impl std::fmt::Debug },
            quote! {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {#body_debug; write!(f, "{}", repr)}
            },
        ),
    };

    let res = if !ty_trait.is_empty() {
        quote! {
            #ty_trait for #ident {
                #ty_fn
            }
        }
    } else {
        quote! {}
    };
    println!("{}\n", res);
    res
}

fn generate_fmt_impl_for_struct(
    data_struct: &syn::DataStruct,
    name: &Ident,
    is_repr: bool,
) -> proc_macro2::TokenStream {
    let fields = &data_struct.fields;
    let fields = fields
        .iter()
        .filter(|f| {
            !f.attrs.iter().any(|attr| {
                let namespace = if is_repr {
                    ATTR_NAMESPACE_REPR
                } else {
                    ATTR_NAMESPACE_STR
                };
                let mut is_skip = matches!(f.vis, Visibility::Public(_));

                if attr.path().is_ident(namespace) {
                    // only parse ATTR_NAMESPACE and not [serde] or [default]
                    attr.parse_nested_meta(|meta| {
                        is_skip = meta.path.is_ident("skip");
                        Ok(())
                    })
                    .unwrap();
                } else if attr.path().is_ident(ATTR_NAMESPACE_NO_SKIP) {
                    is_skip = false;
                }
                is_skip
            })
        })
        .collect::<Vec<_>>();
    let field_fmts = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let postfix = if i + 1 < fields.len() { ", " } else { "" };
            let formatter = if is_repr { "{}={:?}{}" } else { "{}={}{}" };
            match &field.ident {
                Some(ident) => {
                    quote! {
                        repr += &format!(#formatter, stringify!(#ident), self.#ident, #postfix);
                    }
                }
                None => {
                    // If the field doesn't have a name, we generate a name based on its index
                    let index = syn::Index::from(i);
                    quote! {
                        repr += &format!(#formatter stringify!(#index), self.#index, #postfix);
                    }
                }
            }
        })
        .collect::<Vec<_>>();
    quote! {
        let mut repr = "".to_string();
        repr += &format!("{}(", stringify!(#name));
        #(#field_fmts)*
        repr += ")";
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
            let list_str = attr.lit_str ;
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
    string_formater: Option<&Vec<Attribute>>,
) -> proc_macro2::TokenStream {
    let variants = data_enum.variants.iter().collect::<Vec<_>>();
    let mut ident_formatter =  quote! { "{}."} ;
    if let Some(attrs) = string_formater {
        for attr in attrs {
            if attr.path().is_ident("auto_display") {
                if let Some(formatter) = find_display_attribute(attr) {
                    ident_formatter = formatter;
                    println!("Found parent formatter: {:?}", ident_formatter.clone());
                    break;
                } 
                break;
            }
        }
    }

    let fmt_str = ident_formatter.to_string();

    // Check if the formatter string contains "{}"
    let ident_formatter = if fmt_str.contains("{}") {
        quote! {
            &format!(#ident_formatter, stringify!(#name))
        }
    } else {
        quote! {
            &format!("{}", #ident_formatter)
        }
    };


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
                if attr.path().is_ident(namespace) {
                    attr.parse_nested_meta(|meta| {
                        if meta.path.is_ident("skip") {
                            to_skip = true;
                        }
                        Ok(())
                    }).unwrap();
                }
                if attr.path().is_ident("auto_display") {
                    display_attr = Some(attr);
                }
            }
        
            (to_skip, display_attr)
        };
        
        let mut variant_fmt = quote! { "{}"};
        if let Some(display_attr) = display_attr {
            if let Some(formatter) = find_display_attribute(display_attr) {
                println!("Found variant formatter: {:?}", formatter.clone());
                variant_fmt = formatter;
            }
        }



        // If {} is not in ident_fmt, we must not format ident.
        // If {} is not in variant_fmt, we don't use stringify! either
        match &variant.fields {
            Fields::Unit => {
                // Check if the formatter string contains "{}"
                let variant_fmt = if variant_fmt.to_string().contains("{}") {
                    quote! {
                        &format!(#variant_fmt, stringify!(#ident))
                    }
                } else {
                    quote! {
                        &format!("\"{}\"", #variant_fmt)
                    }
                };
                if !to_skip {
                    quote! {
                        Self::#ident => repr += #variant_fmt,
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
                    let quote = 
                    quote! { #name::#ident(ref single) => {#ident_formatter;} };
                    quote
                } else {
                    quote! {
                        #ident => repr += "<variant skipped>",
                    }
                }  // TODO now that we have AutoDisplay we want this
            }
            Fields::Named(fields) => {
                let field_names: Vec<_> = fields.named.iter().map(|f| &f.ident).collect();
                let mut format_string = "{}.{}(".to_string();
                let formatter = if is_repr { "{:?}" } else { "{}"};
                for (i, name) in field_names.iter().enumerate() {
                    if i == 0 {
                            format_string = format!("{format_string}{}={formatter}", name.as_ref().unwrap());
                    } else {
                        format_string = format!("{format_string}, {}={formatter}", name.as_ref().unwrap());
                    }
                }
                format_string = format!("{format_string})");
                if !to_skip {
                    let mut names = Vec::new();
                    for name in field_names.clone() {
                        names.push(quote! { #name });
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
            _ => {
                // Default case: stringify the variant name
                quote! {  &format!("{}", stringify!(#ident)); }
            }
        }
    }).collect::<Vec<_>>();

    quote! {
        let mut repr = "".to_string();
        repr += #ident_formatter;
        match self {
            #(#arms)*
        }
    }
}
