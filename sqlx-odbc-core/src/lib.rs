//! SQLx ODBC driver.
//!
//! This crate provides an ODBC backend for SQLx, allowing connections to any
//! ODBC-compatible database.
//!
//! ## Example
//!
//! ```rust,no_run
//! use sqlx_odbc::odbc::{OdbcConnectOptions, OdbcConnection};
//! use sqlx_core::connection::Connection;
//!
//! # async fn example() -> Result<(), sqlx_core::Error> {
//! let options = OdbcConnectOptions::new("DSN=MyDSN;UID=user;PWD=pass");
//! let mut conn = OdbcConnection::establish(&options).await?;
//! # Ok(())
//! # }
//! ```

pub mod odbc;

// Re-export main types at crate root for convenience
pub use odbc::{
    Odbc, OdbcArguments, OdbcColumn, OdbcConnection, OdbcConnectOptions, OdbcDatabaseError,
    OdbcExecutor, OdbcPool, OdbcPoolOptions, OdbcQueryResult, OdbcRow, OdbcStatement,
    OdbcTransactionManager, OdbcTypeInfo, OdbcValue, OdbcValueRef,
};

// Re-export sqlx_core for downstream use
pub use sqlx_core;
