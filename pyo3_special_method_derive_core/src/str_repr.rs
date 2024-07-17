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

    let debug = quote!{
        let mut repr = "".to_string();
        repr += &format!("{}(", stringify!(#ident));
        #body_debug
        repr += ")";
    };

    let fmt =  quote!{
        let mut repr = "".to_string();
        repr += &format!("{}(", stringify!(#ident));
        #body_display
        repr += ")"; 
    };



    // Determine which traits to implement
    let (ty_trait, ty_fn) = match ty {
        DeriveType::ForAutoDisplay => (
            quote! { impl std::fmt::Display },
            quote! {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {#fmt; write!(f, "{}", repr)}
            },
        ),
        DeriveType::ForAutoDebug => (
            quote! { impl std::fmt::Debug },
            quote! {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {#debug; write!(f, "{}", repr)}
            }
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
    is_repr: bool,
) -> proc_macro2::TokenStream {
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
            let formatter = if is_repr { "{}={:?}{}" } else {  "{}={}{}" };
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
    // Collect the mapped tokens into a TokenStream
    quote!{#(#field_fmts)*}
}

fn generate_fmt_impl_for_enum(
    data_enum: &syn::DataEnum,
    name: &Ident,
    is_repr: bool,
) ->proc_macro2::TokenStream {
    let variants = data_enum.variants.iter().collect::<Vec<_>>();
    let arms = variants.iter()
        .map(|variant| {
            let ident = &variant.ident;
            let to_skip = variant.attrs.iter().any(|attr| {
                let mut is_skip = false;
                let namespace = if is_repr {
                    ATTR_NAMESPACE_REPR
                } else {
                    ATTR_NAMESPACE_STR
                };               
                if attr.path().is_ident(namespace) { // only parse ATTR_NAMESPACE and not [serde] or [default]
                    attr.parse_nested_meta(|meta| {
                        is_skip = meta.path.is_ident("skip");
                        Ok(())
                    })
                    .unwrap();
                }
                is_skip    
            }
            );
            println!("Processing {}, {}", name, ident);
            match &variant.fields {
                Fields::Unit => {
                    if !to_skip {
                        quote! {
                            Self::#ident => repr += &format!("{}", stringify!(#ident)),
                        }
                    } else {
                        quote! {
                            Self::#ident => repr += "<variant skipped>",
                        }
                    } 
                    // potantially for a more pythonic print, {}.{} replace by just {}. Ex: PrependScheme.First -> "first"
                    // as in most cases, we have something like Class(adress = ) and don't want Adress.Dummy, but just "dummy"
                    // Maybe if Str then we have "{}" but AutoDisplay is rust so "{}.{}".
                }
                syn::Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                    // Tuple variant with one field
                    if !to_skip {
                        let quote = 
                        quote! { #name::#ident(ref single) => {&format!("{}", single);} };
                        print!("{}\n",quote);
                        quote
                    } else {
                        println!("Skipping {}, {}", ident, name);
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
        match self {
            #(#arms)*
        }
    }
}
