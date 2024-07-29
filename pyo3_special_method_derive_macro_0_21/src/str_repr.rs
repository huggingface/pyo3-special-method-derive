use std::collections::btree_map::Keys;

use crate::ATTR_NAMESPACE_FORMATTER;
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Error, Field, Fields, Ident, LitStr, Token, Visibility};


const DEFAULT_ENUM_IDENT_FORMATTER: &str = "{}.{}";
const DEFAULT_ELEMENT_FORMATTER: &str = "{}={}";
const DEFAULT_STRUCT_IDENT_FORMATTER: &str = "{}({})";

pub(crate) enum DeriveType {
    ForAutoDisplay,
    ForAutoDebug,
}


// Define a struct to hold the parsed tokens (fmt="{}.{}")
struct FmtAttribute {
    ident: Option<Ident>,
    _eq_token: Option<Token![=]>,
    pub lit_str: Option<LitStr>,
}

// Implement parsing for the FmtAttribute struct
impl Parse for FmtAttribute {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = match input.parse() {
            Ok(ident) => ident,
            Err(_) => None, // skip #[format]
        };
        let _eq_token = match input.parse() {
            Ok(token) => token,
            Err(_) => None, // skip #[format(skip)]
        };
        let lit_str = match input.parse() {
            Ok(str_format) => str_format,
            Err(_) => None, // skip #[format(skip)]
        }; // if we have xxx = , then we parse it all
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
        Ok(fmt_attr) => match fmt_attr.ident {
            Some(fmt_ident) => {
                if fmt_ident == "skip" {
                    return Ok(None);
                } else if fmt_ident == "fmt" {
                    if let Some(list_str) = fmt_attr.lit_str {
                        return Ok(Some(quote! { #list_str }));
                    }
                }
                Err(syn::Error::new_spanned(
                    attr,
                    "Error parsing, should be either `#[format]`, , `#[format(skip)]` or `#[format(fmt=\"\")]`",
                ))
            }
            _ => Ok(None),
        },
        Err(_) => Ok(Some(quote! {})), // this is #[format], we do want to force formatting
    }
}

// uses `find_display_attribute` to decide whether the field should be skipped or not
pub fn skip_formatting<I>(
    attrs: I,
    default_variant_fmt: &mut TokenStream,
    is_skipped: bool
) -> Result<bool, Error>
where I: IntoIterator<Item = Attribute>,
{
    let mut is_skipped = is_skipped; 
    for attr in attrs {
        if attr.path().is_ident(ATTR_NAMESPACE_FORMATTER) {
            match find_display_attribute(&attr) {
                Ok(Some(formatter)) => {if formatter.to_string()!=""{*default_variant_fmt = formatter}; is_skipped=false},
                Err(error) => return Err(error),
                Ok(None)=>{is_skipped = true}, // this is where we skip
            }
            break
        }
    };
    Ok(is_skipped)
}

// Extract the string that should be used to format each fields. "{}", or "MyStruct", or "{}.{}"
// By calling `find_display_attribute` whenever `#[format(fmt="")` is found as attr of a field.
// If a field should be skipped, then it simply won't appear in the vec of ident string and usize.
pub fn extract_field_formatters<T>(fields: Vec<&Field>, token: &T, variant_fmt: String, is_enum: bool)  -> Result<(Vec<Option<Ident>>, Vec<String>, Vec<usize>), Error> 
where T: Spanned+std::fmt::Debug{
    let mut ids: Vec<Option<Ident>> = Vec::new();
    let mut format_strings: Vec<String> = Vec::new();
    let mut formatters_counts: Vec<usize> = Vec::new();
    let mut default_variamt_fmt =  quote!{ #variant_fmt };
    for field in fields{
        let mut visibility = matches!(field.vis, Visibility::Public(_));
        if !is_enum{
            visibility = true;
        }
        print!("Visibility:{}", visibility);
        match skip_formatting(field.attrs.clone(), &mut default_variamt_fmt, !visibility) { 
            Ok(is_skipped) => {
                
                if !is_skipped{
                    print!("Not skipped: {:?}", field.ident.clone());
                    let formatter_str = default_variamt_fmt.to_string();
                    let formatters = formatter_str.matches("{}").count() - formatter_str.matches("{{}}").count();
                    if formatters > 2 {
                        return Err(syn::Error::new(token.clone().span(), "You can specify at most 2 formatters, one for the field name, and one for it's string representation"));
                    };
                    ids.push(field.ident.clone());
                    println!("{}, ids: {:?}\n", formatter_str, field.ident.clone());
                    format_strings.push(formatter_str);
                    formatters_counts.push(formatters);
                } else {
                    print!("\tSkipped: {:?}, token: {:?}\n", field.ident.clone(), token);
                }
            },
            Err(e) =>  return Err(e),
        }
    }
    Ok((ids, format_strings, formatters_counts))
}



fn generate_fmt_impl_for_enum(
    data_enum: &syn::DataEnum,
    name: &Ident,
    is_repr: bool,
    string_formatter: Option<&Vec<Attribute>>,
) -> syn::Result<proc_macro2::TokenStream> {
    let formatter = if is_repr {
        quote! { fmt_debug }
    } else {
        quote! { fmt_display }
    };
    let mut ident_formatter = quote! { #DEFAULT_ENUM_IDENT_FORMATTER };
    if let Some(attrs) = string_formatter {
        match skip_formatting(attrs.clone(), &mut ident_formatter, false) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    }
    let variants = data_enum.variants.iter().collect::<Vec<_>>();
    let arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        match &variant.fields {
            Fields::Unit => {
                let mut default_variamt_fmt =  quote! { #DEFAULT_ELEMENT_FORMATTER};
                let field = variant;
                let extracted_field_names = match skip_formatting(field.attrs.clone(), &mut default_variamt_fmt, false) { 
                    Ok(is_skipped) => {
                        if !is_skipped{
                            let formatter_str = default_variamt_fmt.to_string();
                            let formatters = formatter_str.matches("{}").count() - formatter_str.matches("{{}}").count();
                            if formatters > 2 {
                                return Err(syn::Error::new(variant.span(), "You can specify at most 2 formatters, one for the field name, and one for it's string representation"));
                            };
                            let token_stream = match formatters {
                                    1 => quote!{ #name.#formatter() }, 
                                    2 => quote!{ stringify!(#variant_name), #name.#formatter() },
                                    _ => quote!{},
                            };
                                
                            Ok(quote! {
                                Self::#variant_name => repr += &format!(#formatter_str, #token_stream),
                            })
                        } else {
                            Ok(quote!{})
                        }
                    },
                    Err(e) =>  return Err(e),
                };
                extracted_field_names
            }
            syn::Fields::Unnamed(fields) => {
                println!("Unamed");
                let extracted_field_names = extract_field_formatters(fields.unnamed.iter().collect::<Vec<_>>(), &data_enum.enum_token, DEFAULT_ELEMENT_FORMATTER.to_string(), true);
                match extracted_field_names {
                    Ok((ids, format_strings, formatters_counts)) => {
                        let field_arm = {
                            let token_streams: Vec<TokenStream> = formatters_counts.into_iter().zip(&ids).map(| (n_formatters, name)| {
                                match n_formatters {
                                    1 => quote!{ ,#name.#formatter() }, 
                                    2 => quote!{ ,stringify!(#name), #name.#formatter() },
                                    _ => quote!{},
                                }
                            }).collect();
                            let field_value = &variant.ident;
                            quote! {
                                #(#name::#field_value(single)  => repr += &format!(#format_strings #token_streams)),*,
                            }
                        };
                        Ok(field_arm)
                    },
                    Err(e) => Err(e),
                }
            }
            Fields::Named(fields) => {
                println!("Named");
                let extracted_field_names = extract_field_formatters(fields.named.iter().collect::<Vec<_>>(), &data_enum.enum_token, DEFAULT_ELEMENT_FORMATTER.to_string(), true);
                match extracted_field_names {
                    Ok((ids, format_strings, formatters_counts)) => {
                        let field_arm = {
                            let token_streams: Vec<TokenStream> = formatters_counts.into_iter().zip(&ids).map(| (n_formatters, name)| {
                                match n_formatters {
                                        1 => quote!{ ,#name.#formatter() }, 
                                        2 => quote!{ ,stringify!(#name), #name.#formatter() },
                                        _ => quote!{},
                                    }
                                    
                                }).collect();
                            quote! {
                                #(Self::#variant_name { #ids } => repr += &format!(#format_strings #token_streams)),*,
                            }
                        };
                        Ok(field_arm)
                    },
                    Err(e) => Err(e),
                }
            }
        }
    }).collect::<syn::Result<Vec<_>>>()?;

    // Handle any escaped {}
    let formatters = ident_formatter.to_string().matches("{}").count()
        - ident_formatter.to_string().matches("{{}}").count();

    let token_stream = match formatters {
        0 => quote!{ format!(#ident_formatter) },
        1 => quote!{ format!(#ident_formatter, repr) },
        2 => quote!{ format!(#ident_formatter, stringify!(#name), repr) }, 
        _ => return Err(syn::Error::new(
            data_enum.enum_token.span(),
            "Specify 2 (name, repr), 1 (name), or 0 formatters in the format string.",
        )), 
    };
    println!(
        "Resulting arms: {}",
        arms.iter()
            .map(|t| t.to_string())
            .collect::<Vec<String>>()
            .join("\n")
    );

    Ok(quote! {
        let mut repr = "".to_string();
        match self {
            #(#arms)*
        }
        let repr = #token_stream;
    })
}


fn generate_fmt_impl_for_struct(
    data_struct: &syn::DataStruct,
    name: &Ident,
    is_repr: bool,
    string_formatter: Option<&Vec<Attribute>>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut ident_formatter = quote! { #DEFAULT_STRUCT_IDENT_FORMATTER };
    if let Some(attrs) = string_formatter {
        match skip_formatting(attrs.clone(), &mut ident_formatter, false) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    }
    let fields = data_struct.fields.iter().collect::<Vec<_>>();
    let formatter = if is_repr { quote! { fmt_debug } } else { quote! { fmt_display } };
    let extracted_field_names = extract_field_formatters( fields, &data_struct.struct_token, DEFAULT_ELEMENT_FORMATTER.to_string(), false);
    let field_arms = match extracted_field_names {
        Ok((ids, format_strings, formatters_counts)) => {
            let field_arm = {
                let token_streams: Vec<TokenStream> = formatters_counts.into_iter().zip(&ids).enumerate().map(| (idx, (n_formatters, name))| {
                    let ident = match name {
                        Some(ident) => quote! { #ident },
                        None => {let ident = syn::Index::from(idx); quote!(#ident)},
                    };
                    let token_stream = match n_formatters {
                            1 => quote!{ ,self.#ident.#formatter() }, 
                            2 => quote!{ ,stringify!(#ident), self.#ident.#formatter()},
                            _ => quote!{},
                        };
                        token_stream
                    }).collect();
                quote! {
                    // TODO name should be the variant fmt here
                    #(repr += &format!(#format_strings #token_streams)),*;
                }
            };
            Ok(field_arm)
        },
        Err(e) => Err(e),
    }?;
    println!(
        "field arms: {}",
        field_arms.to_string()
    );

    let formatters = ident_formatter.to_string().matches("{}").count()
    - ident_formatter.to_string().matches("{{}}").count();

    let token_stream = match formatters {
        0 => quote!{ format!(#ident_formatter) },
        1 => quote!{ format!(#ident_formatter, repr) },
        2 => quote!{ format!(#ident_formatter, stringify!(#name), repr) }, 
        _ => return Err(syn::Error::new(
            data_struct.struct_token.span(),
            "Specify 2 (name, repr), 1 (name), or 0 formatters in the format string.",
        )), 
    };

    Ok(quote! {
        let mut repr = "".to_string();
        #field_arms

        let repr = #token_stream;
    })
}


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
