//! Microsoft SQL Server specific extensions for the ODBC driver.
//!
//! This module provides MSSQL-specific functionality that can be optionally
//! enabled by importing the appropriate extension traits.
//!
//! ## Features
//!
//! - `mssql-migrate`: Enables migration support for SQL Server databases.
//!
//! ## Usage
//!
//! To use MSSQL migration support, import the extension trait:
//!
//! ```rust,ignore
//! use sqlx_odbc::odbc::mssql::MssqlMigrateExt;
//! use sqlx_odbc::odbc::mssql::MssqlMigrateDatabaseExt;
//! ```

#[cfg(feature = "mssql-migrate")]
mod migrate;

#[cfg(feature = "mssql-migrate")]
pub use migrate::{MssqlMigrateExt, MssqlMigrateDatabaseExt};
