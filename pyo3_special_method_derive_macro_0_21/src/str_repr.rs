use crate::{ATTR_NAMESPACE_FORMATTER, ATTR_SKIP_NAMESPACE, SKIP_ALL};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Error, Field, Fields, Ident, LitStr, Token, Visibility};
use regex::Regex;

const DEFAULT_ENUM_IDENT_FORMATTER: &str = "{}.{}";
const DEFAULT_ENUM_UNIT_IDENT_FORMATTER: &str = "{}";
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
            _ => Err(syn::Error::new_spanned(
                attr,
                "You have to specify fmt = ...",
            )),
        },
        Err(_) => Ok(Some(quote! {})), // this is #[format], we do want to force formatting
    }
}

// uses `find_display_attribute` to decide whether the field should be skipped or not
pub fn skip_formatting<I>(
    attrs: I,
    default_variant_fmt: &mut String,
    macro_name: &str,
    is_skipped: bool,
) -> Result<bool, Error>
where
    I: IntoIterator<Item = Attribute>,
{
    let mut is_skipped = is_skipped;
    for attr in attrs {
        if attr.path().is_ident(ATTR_NAMESPACE_FORMATTER) {
            match find_display_attribute(&attr) {
                Ok(Some(formatter)) => {
                    if formatter.to_string() != "" {
                        *default_variant_fmt = formatter.to_string().replace('\"', "");
                    };
                    is_skipped = false
                }
                Err(error) => return Err(error),
                Ok(None) => is_skipped = true, // this is where we skip
            }
        } else if attr.path().is_ident(ATTR_SKIP_NAMESPACE)  {
            let _ = attr.parse_nested_meta(|meta| {
                is_skipped |= meta.path.is_ident(macro_name) || meta.path.is_ident(SKIP_ALL);
                Ok(())
            });
        }
    }
    Ok(is_skipped)
}

// Extract the string that should be used to format each fields. "{}", or "MyStruct", or "{}.{}"
// By calling `find_display_attribute` whenever `#[format(fmt="")` is found as attr of a field.
// If a field should be skipped, then it simply won't appear in the vec of ident string and usize.
#[allow(clippy::type_complexity)]
pub fn extract_field_formatters<T>(
    fields: Punctuated<Field, syn::token::Comma>,
    token: &T,
    variant_fmt: String,
    macro_name: &str,
    is_enum: bool,
) -> Result<(Vec<Option<Ident>>, Vec<String>, Vec<usize>), Error>
where
    T: Spanned + std::fmt::Debug,
{
    let mut ids: Vec<Option<Ident>> = Vec::new();
    let mut format_strings: Vec<String> = Vec::new();
    let mut formatters_counts: Vec<usize> = Vec::new();
    let mut default_variamt_fmt = variant_fmt;
    let mut is_first = true;
    for field in fields {
        let mut visibility = matches!(field.vis, Visibility::Public(_));
        if is_enum {
            visibility = true;
        }

        match skip_formatting(field.attrs.clone(), &mut default_variamt_fmt, macro_name,!visibility) {
            Ok(is_skipped) => {
                if !is_skipped {
                    let mut formatter_str = default_variamt_fmt.clone();
                    let formatters =
                        formatter_str.matches("{}").count() - formatter_str.matches("{{}}").count();

                    if !is_first {
                        formatter_str.push_str(", ")
                    } else {
                        is_first = false
                    }
                    if formatters > 2 {
                        return Err(syn::Error::new(token.span(), "You can specify at most 2 formatters, one for the field name, and one for it's string representation"));
                    };
                    ids.push(field.ident.clone());
                    format_strings.push(formatter_str);
                    formatters_counts.push(formatters);
                }
            }
            Err(e) => return Err(e),
        }
    }
    Ok((ids, format_strings, formatters_counts))
}

fn generate_fmt_impl_for_enum(
    data_enum: &syn::DataEnum,
    name: &Ident,
    string_formatter: Option<&Vec<Attribute>>,
    macro_name: &str,
    is_repr: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let formatter = if is_repr {
        quote! { fmt_debug }
    } else {
        quote! { fmt_display }
    };
    let mut enum_formatter = "{}.{}({})".to_string(); // by default pub enum A {...} we show A...
    // check if the user overwrites the enum_formatter: 
    if let Some(attrs) = string_formatter {
        match skip_formatting(attrs.clone(), &mut enum_formatter, macro_name, false) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    }
    // count the number of formatters
    let formatters = enum_formatter.matches("{}").count() - enum_formatter.matches("{{}}").count();
    if formatters==3 && enum_formatter != "{}.{}({})" {
        return Err(syn::Error::new(name.span(), "You can only specify 0, 1 or 2 `{{}}` formatter at the top of an enum"));
    }
    let default_variant_fmt = match formatters {
        0 => {return Ok(quote! { let mut repr = #enum_formatter;})}, // The user does wants to display "MyEnumOnly"
        1 => {"{}".to_string()},           // The user wants to display "MyEnumOnly.{}",
        2 => {"{}({})".to_string()},           // The user wants to display "MyCustom.{}({}}", enum's name and {}
        3 => {"{}({})".to_string()}            // this should only be the default setting "{}.{}({})" which is default
        _ => {return Err(syn::Error::new(name.span(), 
        "You can specify at most 2 formatters at the top of an enum. One for the enum name, one for the variant name, one for the variant's field.
        Something like `#[format[fmt=\"nice_{}.cool_{}[--{}--]\")` which will display enum A{B(C)} as nice_A.cool_B[--C.fmt_display()--]."
        ))}
    };
    if formatters < 3 { // we are not using the default name of the enum
        let re = Regex::new(r"^(.*?)\{").unwrap();
        if let Some(captures) = re.captures(&enum_formatter) {
            if let Some(matched) = captures.get(1) {
                 if matched.as_str().len() > 1 { // we fetch the formatter for the enume to do format!("MyFormat.{}", repr)
                    enum_formatter = matched.as_str().to_string();
                }
            }
        }
    }else {
        enum_formatter=String::from("{}");
    };
    println!("Enum formatter: {} \t default variant fmt {}", enum_formatter, default_variant_fmt);
    let variants = data_enum.variants.iter().collect::<Vec<_>>();
    let arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident; // struct A{ UnitVariantName, NamedVariantName{named:i32}, UnamedVariantNamed(String)}

        match &variant.fields {
            Fields::Unit => { // All of the variants are Unit
                let mut default_unit_fmt =  "{}".to_string(); // Unit fields should always behave like this!
                let field = variant;
                let extracted_field_names = match skip_formatting(field.attrs.clone(), &mut default_unit_fmt,macro_name, false) {
                    Ok(is_skipped) => {
                        if !is_skipped{
                            let formatter_str = default_variant_fmt.clone();
                            let formatters = formatter_str.matches("{}").count() - formatter_str.matches("{{}}").count();
                            let token_stream = match formatters {
                                    0 => quote!{},
                                    1 => quote!{ ,stringify!(#variant_name)},
                                    _ => return Err(syn::Error::new(variant.span(), "You can specify at most 1 formatters,for unit fields of enums")),
                            };
                            Ok(quote! {
                                Self::#variant_name => repr += &format!(#formatter_str  #token_stream),
                            })
                        } else {
                            Ok(quote!{})
                        }
                    },
                    Err(e) =>  return Err(e),
                };
                extracted_field_names
            }
            syn::Fields::Unnamed(_fields) => { // Variants are Unit or Unnamed
                let field_value = &variant.ident;
                let mut default_unamed_fmt = default_variant_fmt.clone();
                let extracted_field_names = match skip_formatting(variant.attrs.clone(), &mut default_unamed_fmt, macro_name, false) {
                    Ok(is_skipped) => {
                        if !is_skipped{
                            let formatter_str = default_unamed_fmt;
                            let formatters = formatter_str.matches("{}").count() - formatter_str.matches("{{}}").count();
                            let token_stream  = match formatters {
                                    0 => quote!{},
                                    1 => quote!{ ,single.#formatter() },
                                    2 => quote!{ ,stringify!(#field_value), single.#formatter() },
                                    _ => return Err(syn::Error::new(variant.span(), "You can specify at most 1 formatters,for unit fields of enums")),
                            };
                            Ok(quote! {
                                #name::#field_value(single)  => repr += &format!(#formatter_str #token_stream),
                            })
                        } else {
                            Ok(quote!{})
                        }
                    },
                    Err(e) =>  return Err(e),
                };
                extracted_field_names
            }
            Fields::Named(fields) => {
                let extracted_field_names = extract_field_formatters(fields.named.clone() , &data_enum.enum_token, default_variant_fmt.clone(), macro_name, true);
                match extracted_field_names {
                    Ok((ids, format_strings, formatters_counts)) => {
                        Ok({
                            let token_streams: Vec<TokenStream> = formatters_counts.into_iter().zip(&ids).map(| (n_formatters, name)| {
                                match n_formatters {
                                        1 => quote!{ ,#name.#formatter() },
                                        2 => quote!{ ,stringify!(#name), #name.#formatter() },
                                        _ => quote!{},
                                    }
                                }).collect();
                            quote! {
                                // For each arm format is gonna be hard to specify no?
                                Self::#variant_name {#(#ids,)*} => repr += &format!(concat!(#(#format_strings, "", )*) #(#token_streams)*),
                            }
                        })
                    },
                    Err(e) => Err(e),
                }
            }
        }
    }).collect::<syn::Result<Vec<_>>>()?;

    let token_stream = quote! { format!(#enum_formatter, repr)};

    let final_stream = quote! {
        let mut repr = "".to_string();
        match self {
            #(#arms)*
        }
        let repr = #token_stream;
    };
    Ok(final_stream)
}

fn generate_fmt_impl_for_struct(
    data_struct: &syn::DataStruct,
    name: &Ident,
    string_formatter: Option<&Vec<Attribute>>,
    macro_name: &str,
    is_repr: bool,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut struct_formatter = "{}({})".to_string();
    let formatter = if is_repr {
        quote! { fmt_debug }
    } else {
        quote! { fmt_display }
    };
    if let Some(attrs) = string_formatter {
        match skip_formatting(attrs.clone(), &mut struct_formatter,macro_name, false) {
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    }

    let formatters = struct_formatter.matches("{}").count()
        - struct_formatter.matches("{{}}").count();
    if formatters == 0 {
        return Ok(quote! { let mut repr = #struct_formatter;});
    }

    let field_arms = match &data_struct.fields {
        Fields::Named(fields) => {
            let extracted_field_names = extract_field_formatters(
                fields.named.clone(),
                &data_struct.struct_token,
                DEFAULT_ELEMENT_FORMATTER.to_string(),
                macro_name,
                false,
            );
            match extracted_field_names {
                Ok((ids, format_strings, formatters_counts)) => Ok({
                    let token_streams: Vec<TokenStream> = formatters_counts
                        .into_iter()
                        .zip(&ids)
                        .enumerate()
                        .map(|(idx, (n_formatters, name))| {
                            let ident = match name {
                                Some(ident) => quote! { #ident },
                                None => {
                                    let ident = syn::Index::from(idx);
                                    quote!(#ident)
                                }
                            };
                            match n_formatters {
                                1 => quote! { ,self.#ident.#formatter() },
                                2 => quote! { ,stringify!(#ident), self.#ident.#formatter()},
                                _ => quote! {},
                            }
                        })
                        .collect();
                    quote! {
                        #(repr += &format!(#format_strings #token_streams));*;
                    }
                }),
                Err(e) => Err(e),
            }
        }
        Fields::Unnamed(_unnamed_fields) => Ok(quote! {}),
        Fields::Unit => Ok(quote! {}),
    }?;

    let token_stream = match formatters {
        0 => quote! { format!(#struct_formatter) },
        1 => quote! { format!(#struct_formatter, repr) },
        2 => quote! { format!(#struct_formatter, stringify!(#name), repr) },
        _ => {
            return Err(syn::Error::new(
                data_struct.struct_token.span(),
                "Specify 2 (name, repr), 1 (name), or 0 formatters in the format string.",
            ))
        }
    };

    let final_stream = quote! {
        let mut repr = "".to_string();
        #field_arms

        let repr = #token_stream;
    };
    Ok(final_stream)
}

macro_rules! create_body {
    ($input:expr, $ident:expr, $is_repr:expr, $name:expr) => {
        match &$input.data {
            syn::Data::Struct(s) => {
                generate_fmt_impl_for_struct(s, $ident, Some(&$input.attrs), $name, $is_repr)
            }
            syn::Data::Enum(e) => {
                generate_fmt_impl_for_enum(e, $ident, Some(&$input.attrs), $name, $is_repr)
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
    name: &str
) -> syn::Result<proc_macro2::TokenStream> {
    // Get the name of the struct
    let ident = &input.ident;
    // Determine if the implementation is for a "repr" type
    let is_repr = matches!(ty, DeriveType::ForAutoDebug);

    // Create body for display and debug
    let body_display = create_body!(input, ident,is_repr, name)?;
    let body_debug = create_body!(input, ident, is_repr, name)?;

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
