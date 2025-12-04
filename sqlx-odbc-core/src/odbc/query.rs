//! Query helpers for ODBC.
//!
//! This module provides convenient functions for building queries,
//! similar to `sqlx::query`, `sqlx::query_as`, etc.

use crate::odbc::{Odbc, OdbcArguments, OdbcRow};
use sqlx_core::from_row::FromRow;

// Re-export query types from sqlx_core
pub use sqlx_core::query::Query;
pub use sqlx_core::query_as::QueryAs;
pub use sqlx_core::query_builder::QueryBuilder;
pub use sqlx_core::query_scalar::QueryScalar;

/// Create a new SQL query for the ODBC database.
///
/// # Example
///
/// ```rust,ignore
/// use sqlx_odbc::query;
///
/// let q = query("SELECT * FROM users WHERE id = ?");
/// ```
pub fn query(sql: &str) -> Query<'_, Odbc, OdbcArguments<'_>> {
    sqlx_core::query::query(sql)
}

/// Create a new SQL query with arguments for the ODBC database.
///
/// # Example
///
/// ```rust,ignore
/// use sqlx_odbc::query_with;
///
/// let args = OdbcArguments::new();
/// let q = query_with("SELECT * FROM users WHERE id = ?", args);
/// ```
pub fn query_with<'q>(sql: &'q str, args: OdbcArguments<'q>) -> Query<'q, Odbc, OdbcArguments<'q>> {
    sqlx_core::query::query_with(sql, args)
}

/// Create a new SQL query that maps results to a type implementing `FromRow`.
///
/// # Example
///
/// ```rust,ignore
/// use sqlx_odbc::query_as;
///
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// let q = query_as::<User>("SELECT id, name FROM users");
/// ```
pub fn query_as<'q, O>(sql: &'q str) -> QueryAs<'q, Odbc, O, OdbcArguments<'q>>
where
    O: for<'r> FromRow<'r, OdbcRow>,
{
    sqlx_core::query_as::query_as(sql)
}

/// Create a new SQL query with arguments that maps results to a type implementing `FromRow`.
///
/// # Example
///
/// ```rust,ignore
/// use sqlx_odbc::query_as_with;
///
/// #[derive(FromRow)]
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// let args = OdbcArguments::new();
/// let q = query_as_with::<User>("SELECT id, name FROM users WHERE id = ?", args);
/// ```
pub fn query_as_with<'q, O>(
    sql: &'q str,
    args: OdbcArguments<'q>,
) -> QueryAs<'q, Odbc, O, OdbcArguments<'q>>
where
    O: for<'r> FromRow<'r, OdbcRow>,
{
    sqlx_core::query_as::query_as_with(sql, args)
}

/// Create a new SQL query that returns a single scalar value.
///
/// # Example
///
/// ```rust,ignore
/// use sqlx_odbc::query_scalar;
///
/// let q = query_scalar::<i64>("SELECT COUNT(*) FROM users");
/// ```
pub fn query_scalar<'q, S>(sql: &'q str) -> QueryScalar<'q, Odbc, S, OdbcArguments<'q>>
where
    (S,): for<'r> FromRow<'r, OdbcRow>,
{
    sqlx_core::query_scalar::query_scalar(sql)
}

/// Create a new SQL query with arguments that returns a single scalar value.
///
/// # Example
///
/// ```rust,ignore
/// use sqlx_odbc::query_scalar_with;
///
/// let args = OdbcArguments::new();
/// let q = query_scalar_with::<i64>("SELECT COUNT(*) FROM users WHERE active = ?", args);
/// ```
pub fn query_scalar_with<'q, S>(
    sql: &'q str,
    args: OdbcArguments<'q>,
) -> QueryScalar<'q, Odbc, S, OdbcArguments<'q>>
where
    (S,): for<'r> FromRow<'r, OdbcRow>,
{
    sqlx_core::query_scalar::query_scalar_with(sql, args)
}
