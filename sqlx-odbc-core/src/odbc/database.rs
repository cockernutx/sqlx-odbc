//! ODBC database definition.

use crate::odbc::{
    OdbcArguments, OdbcColumn, OdbcConnection, OdbcQueryResult, OdbcRow, OdbcStatement,
    OdbcTransactionManager, OdbcTypeInfo, OdbcValue, OdbcValueRef,
};

pub(crate) use sqlx_core::database::{Database, HasStatementCache};

/// ODBC database driver.
///
/// This driver provides generic ODBC connectivity, allowing SQLx to connect to any
/// ODBC-compatible database.
#[derive(Debug)]
pub struct Odbc;

impl Database for Odbc {
    type Connection = OdbcConnection;

    type TransactionManager = OdbcTransactionManager;

    type Row = OdbcRow;

    type QueryResult = OdbcQueryResult;

    type Column = OdbcColumn;

    type TypeInfo = OdbcTypeInfo;

    type Value = OdbcValue;
    type ValueRef<'r> = OdbcValueRef<'r>;

    type Arguments<'q> = OdbcArguments<'q>;
    type ArgumentBuffer<'q> = Vec<OdbcArgumentValue<'q>>;

    type Statement<'q> = OdbcStatement<'q>;

    const NAME: &'static str = "ODBC";

    const URL_SCHEMES: &'static [&'static str] = &["odbc"];
}

impl HasStatementCache for Odbc {}

/// Owned argument value for ODBC queries.
#[derive(Debug, Clone)]
pub enum OdbcArgumentValue<'q> {
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// 8-bit signed integer
    TinyInt(i8),
    /// 16-bit signed integer
    SmallInt(i16),
    /// 32-bit signed integer
    Int(i32),
    /// 64-bit signed integer
    BigInt(i64),
    /// 32-bit floating point
    Float(f32),
    /// 64-bit floating point
    Double(f64),
    /// Text string (borrowed)
    Text(std::borrow::Cow<'q, str>),
    /// Binary data (borrowed)
    Binary(std::borrow::Cow<'q, [u8]>),
    /// Date value
    Date(odbc_api::sys::Date),
    /// Time value
    Time(odbc_api::sys::Time),
    /// Timestamp value
    Timestamp(odbc_api::sys::Timestamp),
}
