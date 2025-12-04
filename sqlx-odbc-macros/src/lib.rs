//! Procedural macros for sqlx-odbc.
//!
//! This crate provides derive macros and other procedural macros for use with
//! the sqlx-odbc ODBC driver for SQLx.

#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
mod from_row;

#[cfg(feature = "query")]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
mod query;

/// Derive macro for implementing `FromRow` trait.
///
/// This macro generates an implementation of `sqlx_core::from_row::FromRow`
/// for structs, allowing them to be directly constructed from database rows.
///
/// # Example
///
/// ```ignore
/// use sqlx_odbc::FromRow;
///
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     name: String,
///     email: Option<String>,
/// }
/// ```
///
/// # Attributes
///
/// - `#[sqlx(rename = "column_name")]` - Rename the field to match a different column name
/// - `#[sqlx(skip)]` - Skip this field when reading from the row (requires Default)
/// - `#[sqlx(default)]` - Use Default::default() if the column is NULL or missing
/// - `#[sqlx(flatten)]` - Flatten nested structs that also implement FromRow
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
#[proc_macro_derive(FromRow, attributes(sqlx))]
pub fn derive_from_row(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    from_row::expand_derive_from_row(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Internal proc macro for expanding SQL queries.
///
/// This macro is not intended to be used directly. Instead, use the wrapper macros
/// like `query!`, `query_as!`, `query_scalar!`, etc.
#[cfg(feature = "query")]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
#[proc_macro]
pub fn expand_query(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    query::expand_query(input.into())
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
