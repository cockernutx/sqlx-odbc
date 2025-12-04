//! Query macro implementation for ODBC.
//!
//! This module provides the `expand_query` proc macro that powers the
//! `query!`, `query_as!`, `query_scalar!` and related macros.
//!
//! Note: This is a simplified implementation that generates runtime queries.
//! For full compile-time query verification, a more complex implementation
//! would be needed that connects to a database at compile time.

use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{Expr, ExprArray, Ident, LitBool, LitStr, Token, Type};

/// Input for the expand_query macro.
pub struct QueryMacroInput {
    /// The SQL query string
    sql: String,
    /// Span of the source for error reporting
    #[allow(dead_code)]
    src_span: Span,
    /// The record type to map results to
    record_type: RecordType,
    /// Arguments to bind to the query
    arg_exprs: Vec<Expr>,
    /// Whether to perform compile-time type checking
    #[allow(dead_code)]
    checked: bool,
    /// Optional file path if query is from a file
    #[allow(dead_code)]
    file_path: Option<String>,
}

/// The type of record to produce from the query.
enum RecordType {
    /// Map results to a user-provided type
    Given(Box<Type>),
    /// Return a scalar value
    Scalar,
    /// Generate an anonymous record type
    Generated,
}

enum QuerySrc {
    String(String),
    File(String),
}

impl Parse for QueryMacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut query_src: Option<(QuerySrc, Span)> = None;
        let mut args: Option<Vec<Expr>> = None;
        let mut record_type = RecordType::Generated;
        let mut checked = true;

        let mut expect_comma = false;

        while !input.is_empty() {
            if expect_comma {
                let _ = input.parse::<Token![,]>()?;
            }

            let key: Ident = input.parse()?;
            let _ = input.parse::<Token![=]>()?;

            if key == "source" {
                let span = input.span();
                let query_str = Punctuated::<LitStr, Token![+]>::parse_separated_nonempty(input)?
                    .iter()
                    .map(LitStr::value)
                    .collect();
                query_src = Some((QuerySrc::String(query_str), span));
            } else if key == "source_file" {
                let lit_str = input.parse::<LitStr>()?;
                query_src = Some((QuerySrc::File(lit_str.value()), lit_str.span()));
            } else if key == "args" {
                let exprs = input.parse::<ExprArray>()?;
                args = Some(exprs.elems.into_iter().collect());
            } else if key == "record" {
                if !matches!(record_type, RecordType::Generated) {
                    return Err(input.error("colliding `scalar` or `record` key"));
                }
                record_type = RecordType::Given(input.parse()?);
            } else if key == "scalar" {
                if !matches!(record_type, RecordType::Generated) {
                    return Err(input.error("colliding `scalar` or `record` key"));
                }
                // We expect `scalar = _`
                input.parse::<Token![_]>()?;
                record_type = RecordType::Scalar;
            } else if key == "checked" {
                let lit_bool = input.parse::<LitBool>()?;
                checked = lit_bool.value;
            } else {
                let message = format!("unexpected input key: {key}");
                return Err(syn::Error::new_spanned(key, message));
            }

            expect_comma = true;
        }

        let (src, src_span) =
            query_src.ok_or_else(|| input.error("expected `source` or `source_file` key"))?;

        let arg_exprs = args.unwrap_or_default();

        let (sql, file_path) = match src {
            QuerySrc::String(s) => (s, None),
            QuerySrc::File(path) => {
                // For file-based queries, we read the file at compile time
                let resolved = resolve_path(&path, src_span)?;
                let content = std::fs::read_to_string(&resolved).map_err(|e| {
                    syn::Error::new(src_span, format!("failed to read query file: {e}"))
                })?;
                (content, Some(path))
            }
        };

        Ok(QueryMacroInput {
            sql,
            src_span,
            record_type,
            arg_exprs,
            checked,
            file_path,
        })
    }
}

/// Resolve a relative path to an absolute path.
fn resolve_path(path: &str, err_span: Span) -> syn::Result<std::path::PathBuf> {
    let path = std::path::Path::new(path);

    if path.is_absolute() {
        return Ok(path.to_owned());
    }

    // Try to get CARGO_MANIFEST_DIR
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").map_err(|_| {
        syn::Error::new(
            err_span,
            "CARGO_MANIFEST_DIR is not set; cannot resolve relative path",
        )
    })?;

    Ok(std::path::Path::new(&manifest_dir).join(path))
}

/// Expand the query macro input into generated code.
pub fn expand_query(input: TokenStream) -> syn::Result<TokenStream> {
    let input = syn::parse2::<QueryMacroInput>(input)?;

    // Generate the code based on the record type
    let output = match &input.record_type {
        RecordType::Scalar => generate_scalar_query(&input),
        RecordType::Given(out_ty) => generate_query_as(&input, out_ty),
        RecordType::Generated => generate_query(&input),
    };

    Ok(output)
}

/// Generate code for a basic query (returning anonymous rows).
fn generate_query(input: &QueryMacroInput) -> TokenStream {
    let sql = &input.sql;
    let args = &input.arg_exprs;

    if args.is_empty() {
        quote! {
            ::sqlx_odbc::query(#sql)
        }
    } else {
        quote! {
            {
                let mut __query_args = ::sqlx_odbc::OdbcArguments::default();
                #(
                    let _ = __query_args.add(#args);
                )*
                ::sqlx_odbc::query_with(#sql, __query_args)
            }
        }
    }
}

/// Generate code for a query_as (returning a specific type).
fn generate_query_as(input: &QueryMacroInput, out_ty: &Type) -> TokenStream {
    let sql = &input.sql;
    let args = &input.arg_exprs;

    if args.is_empty() {
        quote! {
            ::sqlx_odbc::query_as::<#out_ty>(#sql)
        }
    } else {
        quote! {
            {
                let mut __query_args = ::sqlx_odbc::OdbcArguments::default();
                #(
                    let _ = __query_args.add(#args);
                )*
                ::sqlx_odbc::query_as_with::<#out_ty>(#sql, __query_args)
            }
        }
    }
}

/// Generate code for a scalar query (returning a single value).
fn generate_scalar_query(input: &QueryMacroInput) -> TokenStream {
    let sql = &input.sql;
    let args = &input.arg_exprs;

    if args.is_empty() {
        quote! {
            ::sqlx_odbc::query_scalar(#sql)
        }
    } else {
        quote! {
            {
                let mut __query_args = ::sqlx_odbc::OdbcArguments::default();
                #(
                    let _ = __query_args.add(#args);
                )*
                ::sqlx_odbc::query_scalar_with(#sql, __query_args)
            }
        }
    }
}
