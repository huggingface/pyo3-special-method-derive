use proc_macro2::TokenStream;
use quote::quote;
use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token, Attribute, Expr, Ident
};



pub fn find_format_string(attrs: &[Attribute], ident: Ident) -> Option<String> {
    let attr = attrs.iter().find(|attr| attr.path.is_ident(ident));
    if Some(attr) = attr {
        
    } else {
        None
    }
}