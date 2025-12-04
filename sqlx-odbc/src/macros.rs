#![cfg(feature = "query")]

//! SQL query macros for ODBC.
//!
//! This module provides wrapper macros similar to SQLx's `query!`, `query_as!`, etc.
//! These macros delegate to the proc-macro crate for compile-time query checking.
//!
//! ref: <https://github.com/launchbadge/sqlx/blob/6651d2df72586519708147d96e1ec1054a898c1e/src/macros/mod.rs>

#[doc(hidden)]
pub use sqlx_odbc_macros;

/// Execute a SQL query with compile-time verification.
///
/// This macro is similar to `sqlx::query!` but for ODBC databases.
///
/// See [sqlx::query!](https://docs.rs/sqlx/latest/sqlx/macro.query.html) for details.
///
/// # Example
///
/// ```ignore
/// let rows = sqlx_odbc::query!("SELECT id, name FROM users")
///     .fetch_all(&mut conn)
///     .await?;
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query (
    ($query:expr) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(source = $query)
    });
    ($query:expr, $($args:tt)*) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(source = $query, args = [$($args)*])
    })
);

/// Execute a SQL query without compile-time type checking.
///
/// This macro is similar to `sqlx::query_unchecked!` but for ODBC databases.
///
/// See [sqlx::query_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_unchecked (
    ($query:expr) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(source = $query, checked = false)
    });
    ($query:expr, $($args:tt)*) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(source = $query, args = [$($args)*], checked = false)
    })
);

/// Execute a SQL query from a file with compile-time verification.
///
/// This macro is similar to `sqlx::query_file!` but for ODBC databases.
///
/// See [sqlx::query_file!](https://docs.rs/sqlx/latest/sqlx/macro.query_file.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_file (
    ($path:literal) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(source_file = $path)
    });
    ($path:literal, $($args:tt)*) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(source_file = $path, args = [$($args)*])
    })
);

/// Execute a SQL query from a file without compile-time type checking.
///
/// This macro is similar to `sqlx::query_file_unchecked!` but for ODBC databases.
///
/// See [sqlx::query_file_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_file_unchecked (
    ($path:literal) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(source_file = $path, checked = false)
    });
    ($path:literal, $($args:tt)*) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(source_file = $path, args = [$($args)*], checked = false)
    })
);

/// Execute a SQL query and map results to a custom type.
///
/// This macro is similar to `sqlx::query_as!` but for ODBC databases.
///
/// See [sqlx::query_as!](https://docs.rs/sqlx/latest/sqlx/macro.query_as.html) for details.
///
/// # Example
///
/// ```ignore
/// struct User {
///     id: i32,
///     name: String,
/// }
///
/// let users = sqlx_odbc::query_as!(User, "SELECT id, name FROM users")
///     .fetch_all(&mut conn)
///     .await?;
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_as (
    ($out_struct:path, $query:expr) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(record = $out_struct, source = $query)
    });
    ($out_struct:path, $query:expr, $($args:tt)*) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(record = $out_struct, source = $query, args = [$($args)*])
    })
);

/// Execute a SQL query from a file and map results to a custom type.
///
/// This macro is similar to `sqlx::query_file_as!` but for ODBC databases.
///
/// See [sqlx::query_file_as!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_as.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_file_as (
    ($out_struct:path, $path:literal) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(record = $out_struct, source_file = $path)
    });
    ($out_struct:path, $path:literal, $($args:tt)*) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(record = $out_struct, source_file = $path, args = [$($args)*])
    })
);

/// Execute a SQL query and map results to a custom type without compile-time type checking.
///
/// This macro is similar to `sqlx::query_as_unchecked!` but for ODBC databases.
///
/// See [sqlx::query_as_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_as_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_as_unchecked (
    ($out_struct:path, $query:expr) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(record = $out_struct, source = $query, checked = false)
    });
    ($out_struct:path, $query:expr, $($args:tt)*) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(record = $out_struct, source = $query, args = [$($args)*], checked = false)
    })
);

/// Execute a SQL query from a file and map results to a custom type without compile-time type checking.
///
/// This macro is similar to `sqlx::query_file_as_unchecked!` but for ODBC databases.
///
/// See [sqlx::query_file_as_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_as_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_file_as_unchecked (
    ($out_struct:path, $path:literal) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(record = $out_struct, source_file = $path, checked = false)
    });
    ($out_struct:path, $path:literal, $($args:tt)*) => ({
        $crate::macros::sqlx_odbc_macros::expand_query!(record = $out_struct, source_file = $path, args = [$($args)*], checked = false)
    })
);

/// Execute a SQL query and return a single scalar value.
///
/// This macro is similar to `sqlx::query_scalar!` but for ODBC databases.
///
/// See [sqlx::query_scalar!](https://docs.rs/sqlx/latest/sqlx/macro.query_scalar.html) for details.
///
/// # Example
///
/// ```ignore
/// let count: i64 = sqlx_odbc::query_scalar!("SELECT COUNT(*) FROM users")
///     .fetch_one(&mut conn)
///     .await?;
/// ```
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_scalar (
    ($query:expr) => (
        $crate::macros::sqlx_odbc_macros::expand_query!(scalar = _, source = $query)
    );
    ($query:expr, $($args:tt)*) => (
        $crate::macros::sqlx_odbc_macros::expand_query!(scalar = _, source = $query, args = [$($args)*])
    )
);

/// Execute a SQL query from a file and return a single scalar value.
///
/// This macro is similar to `sqlx::query_file_scalar!` but for ODBC databases.
///
/// See [sqlx::query_file_scalar!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_scalar.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_file_scalar (
    ($path:literal) => (
        $crate::macros::sqlx_odbc_macros::expand_query!(scalar = _, source_file = $path)
    );
    ($path:literal, $($args:tt)*) => (
        $crate::macros::sqlx_odbc_macros::expand_query!(scalar = _, source_file = $path, args = [$($args)*])
    )
);

/// Execute a SQL query and return a single scalar value without compile-time type checking.
///
/// This macro is similar to `sqlx::query_scalar_unchecked!` but for ODBC databases.
///
/// See [sqlx::query_scalar_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_scalar_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_scalar_unchecked (
    ($query:expr) => (
        $crate::macros::sqlx_odbc_macros::expand_query!(scalar = _, source = $query, checked = false)
    );
    ($query:expr, $($args:tt)*) => (
        $crate::macros::sqlx_odbc_macros::expand_query!(scalar = _, source = $query, args = [$($args)*], checked = false)
    )
);

/// Execute a SQL query from a file and return a single scalar value without compile-time type checking.
///
/// This macro is similar to `sqlx::query_file_scalar_unchecked!` but for ODBC databases.
///
/// See [sqlx::query_file_scalar_unchecked!](https://docs.rs/sqlx/latest/sqlx/macro.query_file_scalar_unchecked.html) for details.
#[macro_export]
#[cfg_attr(docsrs, doc(cfg(feature = "query")))]
macro_rules! query_file_scalar_unchecked (
    ($path:literal) => (
        $crate::macros::sqlx_odbc_macros::expand_query!(scalar = _, source_file = $path, checked = false)
    );
    ($path:literal, $($args:tt)*) => (
        $crate::macros::sqlx_odbc_macros::expand_query!(scalar = _, source_file = $path, args = [$($args)*], checked = false)
    )
);
