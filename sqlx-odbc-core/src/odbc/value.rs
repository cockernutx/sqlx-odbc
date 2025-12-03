//! ODBC value types.

use crate::odbc::{Odbc, OdbcTypeInfo};
use sqlx_core::value::{Value, ValueRef};
use std::borrow::Cow;

/// Enum containing an owned value for all supported ODBC types.
#[derive(Debug, Clone)]
pub enum OdbcValueData {
    /// Null value
    Null,
    /// Boolean/Bit value
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
    /// Text string
    Text(String),
    /// Binary data
    Binary(Vec<u8>),
    /// Date value
    Date(odbc_api::sys::Date),
    /// Time value
    Time(odbc_api::sys::Time),
    /// Timestamp value
    Timestamp(odbc_api::sys::Timestamp),
}

/// A reference to a value from an ODBC result.
#[derive(Debug, Clone)]
pub struct OdbcValueRef<'r> {
    pub(crate) data: &'r OdbcValueData,
    pub(crate) type_info: OdbcTypeInfo,
}

impl<'r> OdbcValueRef<'r> {
    /// Create a new value reference
    pub fn new(data: &'r OdbcValueData, type_info: OdbcTypeInfo) -> Self {
        Self { data, type_info }
    }

    /// Get the underlying data
    pub fn data(&self) -> &'r OdbcValueData {
        self.data
    }
}

impl ValueRef<'_> for OdbcValueRef<'_> {
    type Database = Odbc;

    fn to_owned(&self) -> OdbcValue {
        OdbcValue {
            data: self.data.clone(),
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        matches!(self.data, OdbcValueData::Null)
    }
}

/// An owned value from an ODBC result.
#[derive(Debug, Clone)]
pub struct OdbcValue {
    pub(crate) data: OdbcValueData,
    pub(crate) type_info: OdbcTypeInfo,
}

impl OdbcValue {
    /// Create a new owned value
    pub fn new(data: OdbcValueData, type_info: OdbcTypeInfo) -> Self {
        Self { data, type_info }
    }

    /// Create a null value
    pub fn null() -> Self {
        Self {
            data: OdbcValueData::Null,
            type_info: OdbcTypeInfo::null(),
        }
    }

    /// Get the underlying data
    pub fn data(&self) -> &OdbcValueData {
        &self.data
    }

    /// Take ownership of the underlying data
    pub fn into_data(self) -> OdbcValueData {
        self.data
    }
}

impl Value for OdbcValue {
    type Database = Odbc;

    fn as_ref(&self) -> OdbcValueRef<'_> {
        OdbcValueRef {
            data: &self.data,
            type_info: self.type_info.clone(),
        }
    }

    fn type_info(&self) -> Cow<'_, OdbcTypeInfo> {
        Cow::Borrowed(&self.type_info)
    }

    fn is_null(&self) -> bool {
        matches!(self.data, OdbcValueData::Null)
    }
}

// Conversion helpers
impl From<bool> for OdbcValueData {
    fn from(v: bool) -> Self {
        OdbcValueData::Bool(v)
    }
}

impl From<i8> for OdbcValueData {
    fn from(v: i8) -> Self {
        OdbcValueData::TinyInt(v)
    }
}

impl From<i16> for OdbcValueData {
    fn from(v: i16) -> Self {
        OdbcValueData::SmallInt(v)
    }
}

impl From<i32> for OdbcValueData {
    fn from(v: i32) -> Self {
        OdbcValueData::Int(v)
    }
}

impl From<i64> for OdbcValueData {
    fn from(v: i64) -> Self {
        OdbcValueData::BigInt(v)
    }
}

impl From<f32> for OdbcValueData {
    fn from(v: f32) -> Self {
        OdbcValueData::Float(v)
    }
}

impl From<f64> for OdbcValueData {
    fn from(v: f64) -> Self {
        OdbcValueData::Double(v)
    }
}

impl From<String> for OdbcValueData {
    fn from(v: String) -> Self {
        OdbcValueData::Text(v)
    }
}

impl From<&str> for OdbcValueData {
    fn from(v: &str) -> Self {
        OdbcValueData::Text(v.to_string())
    }
}

impl From<Vec<u8>> for OdbcValueData {
    fn from(v: Vec<u8>) -> Self {
        OdbcValueData::Binary(v)
    }
}

impl From<&[u8]> for OdbcValueData {
    fn from(v: &[u8]) -> Self {
        OdbcValueData::Binary(v.to_vec())
    }
}
