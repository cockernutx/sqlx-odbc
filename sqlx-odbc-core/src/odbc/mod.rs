//! ODBC database driver for SQLx.
//!
//! This module provides an ODBC backend for SQLx, allowing connections to any
//! ODBC-compatible database.
//!
//! ## Connection Strings
//!
//! ODBC connections use standard ODBC connection strings:
//!
//! ```text
//! // DSN-based connection
//! DSN=MyDataSource;UID=myuser;PWD=mypassword
//!
//! // Driver-based connection  
//! Driver={ODBC Driver 17 for SQL Server};Server=localhost;Database=test
//!
//! // URL-style (converted internally)
//! odbc://MyDataSource/mydb?UID=user&PWD=pass
//! ```
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

mod arguments;
mod column;
mod connection;
mod database;
mod error;
mod options;
mod query_result;
mod row;
mod statement;
mod transaction;
mod type_info;
pub mod types;
mod value;

/// Microsoft SQL Server specific extensions.
///
/// This module provides MSSQL-specific functionality through extension traits.
/// Enable the `mssql-migrate` feature and import the traits to use them.
#[cfg(feature = "mssql-migrate")]
pub mod mssql;

/// PostgreSQL specific extensions.
///
/// This module provides PostgreSQL-specific functionality through extension traits.
/// Enable the `postgres-migrate` feature and import the traits to use them.
#[cfg(feature = "postgres-migrate")]
pub mod postgres;

// Re-export main types
pub use arguments::OdbcArguments;
pub use column::OdbcColumn;
pub use connection::OdbcConnection;
pub use database::{Odbc, OdbcArgumentValue};
pub use error::OdbcDatabaseError;
pub use options::{OdbcBufferSettings, OdbcConnectOptions};
pub use query_result::OdbcQueryResult;
pub use row::OdbcRow;
pub use statement::{OdbcStatement, OdbcStatementMetadata};
pub use transaction::OdbcTransactionManager;
pub use type_info::{DataTypeExt, OdbcTypeInfo};
pub use value::{OdbcValue, OdbcValueData, OdbcValueRef};

use sqlx_core::executor::Executor;

/// Type alias for an ODBC connection pool.
pub type OdbcPool = sqlx_core::pool::Pool<Odbc>;

/// Type alias for ODBC pool options.
pub type OdbcPoolOptions = sqlx_core::pool::PoolOptions<Odbc>;

/// An alias for [`Executor<'_, Database = Odbc>`][Executor].
pub trait OdbcExecutor<'c>: Executor<'c, Database = Odbc> {}
impl<'c, T: Executor<'c, Database = Odbc>> OdbcExecutor<'c> for T {}
