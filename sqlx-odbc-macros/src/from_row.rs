//! FromRow derive macro implementation.

use heck::ToSnakeCase;
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::{
    spanned::Spanned, Data, DeriveInput, Fields, LitStr,
};

pub fn expand_derive_from_row(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse2(input)?;
    
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "FromRow cannot be derived for tuple structs",
                ))
            }
            Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    &input,
                    "FromRow cannot be derived for unit structs",
                ))
            }
        },
        Data::Enum(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "FromRow cannot be derived for enums",
            ))
        }
        Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                &input,
                "FromRow cannot be derived for unions",
            ))
        }
    };

    let mut field_initializers = Vec::new();
    
    for field in fields {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ty = &field.ty;
        
        let mut column_name = field_ident.to_string().to_snake_case();
        let mut skip = false;
        let mut use_default = false;
        let mut flatten = false;
        
        // Parse attributes
        for attr in &field.attrs {
            if !attr.path().is_ident("sqlx") {
                continue;
            }
            
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    let value = meta.value()?;
                    let lit: LitStr = value.parse()?;
                    column_name = lit.value();
                } else if meta.path.is_ident("skip") {
                    skip = true;
                } else if meta.path.is_ident("default") {
                    use_default = true;
                } else if meta.path.is_ident("flatten") {
                    flatten = true;
                }
                Ok(())
            })?;
        }
        
        let initializer = if skip {
            quote_spanned! { field.span() =>
                #field_ident: ::std::default::Default::default()
            }
        } else if flatten {
            quote_spanned! { field.span() =>
                #field_ident: <#field_ty as ::sqlx_core::from_row::FromRow<'r, R>>::from_row(row)?
            }
        } else if use_default {
            quote_spanned! { field.span() =>
                #field_ident: row.try_get::<#field_ty, _>(#column_name)
                    .unwrap_or_default()
            }
        } else {
            quote_spanned! { field.span() =>
                #field_ident: row.try_get(#column_name)?
            }
        };
        
        field_initializers.push(initializer);
    }
    
    let expanded = quote! {
        impl #impl_generics ::sqlx_core::from_row::FromRow<'r, R> for #name #ty_generics
        #where_clause
        where
            R: ::sqlx_core::row::Row,
        {
            fn from_row(row: &'r R) -> ::std::result::Result<Self, ::sqlx_core::Error> {
                use ::sqlx_core::row::Row;
                Ok(Self {
                    #(#field_initializers),*
                })
            }
        }
    };
    
    Ok(expanded)
}
