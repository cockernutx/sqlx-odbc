//! SQLx ODBC Driver
//!
//! This crate provides an ODBC backend for SQLx, allowing connections to any
//! ODBC-compatible database through a unified async interface.
//!
//! ## Features
//!
//! - **Generic ODBC connectivity** - Connect to any database with an ODBC driver
//! - **Async operations** - Non-blocking database operations using Tokio
//! - **SQLx compatibility** - Full integration with SQLx's type system and traits
//! - **Database-specific migrations** - Optional support for MSSQL and PostgreSQL migrations
//!
//! ## Example
//!
//! ```rust,no_run
//! use sqlx_odbc::{OdbcConnectOptions, OdbcConnection};
//! use sqlx_core::connection::Connection;
//!
//! # async fn example() -> Result<(), sqlx_core::Error> {
//! let options = OdbcConnectOptions::new(
//!     "Driver={ODBC Driver 18 for SQL Server};Server=localhost;Database=test;UID=sa;PWD=password"
//! );
//! let mut conn = OdbcConnection::establish(&options).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ## Feature Flags
//!
//! - `macros` - Enable derive macros (includes `derive`)
//! - `derive` - Enable the `FromRow` derive macro
//! - `mssql-migrate` - Enable Microsoft SQL Server migration support
//! - `postgres-migrate` - Enable PostgreSQL migration support
//! - `serde` - Enable serde serialization support
//! - `offline` - Enable offline mode support

#![cfg_attr(docsrs, feature(doc_cfg))]

// Re-export the FromRow derive macro when the derive feature is enabled
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use sqlx_odbc_macros::FromRow;

// Re-export the FromRow trait from sqlx_core
pub use sqlx_odbc_core::sqlx_core::from_row::FromRow;

// Re-export everything from sqlx-odbc-core
pub use sqlx_odbc_core::*;
