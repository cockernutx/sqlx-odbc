//! PostgreSQL specific extensions for the ODBC driver.
//!
//! This module provides PostgreSQL-specific functionality that can be optionally
//! enabled by importing the appropriate extension traits.
//!
//! ## Features
//!
//! - `postgres-migrate`: Enables migration support for PostgreSQL databases.
//!
//! ## Usage
//!
//! To use PostgreSQL migration support, import the extension trait:
//!
//! ```rust,ignore
//! use sqlx_odbc::odbc::postgres::PostgresMigrateExt;
//! use sqlx_odbc::odbc::postgres::PostgresMigrateDatabaseExt;
//! ```

#[cfg(feature = "postgres-migrate")]
mod migrate;

#[cfg(feature = "postgres-migrate")]
pub use migrate::{PostgresMigrateExt, PostgresMigrateDatabaseExt};
