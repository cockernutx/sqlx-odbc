//! Type implementations for ODBC.
//!
//! This module provides `Type`, `Encode`, and `Decode` implementations for
//! Rust types that can be used with ODBC databases.

use crate::odbc::database::OdbcArgumentValue;
use crate::odbc::{Odbc, OdbcTypeInfo, OdbcValueData, OdbcValueRef};
use sqlx_core::decode::Decode;
use sqlx_core::encode::{Encode, IsNull};
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;

// ============================================================================
// Boolean
// ============================================================================

impl Type<Odbc> for bool {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::bit()
    }

    fn compatible(ty: &OdbcTypeInfo) -> bool {
        matches!(ty.data_type(), odbc_api::DataType::Bit)
    }
}

impl<'q> Encode<'q, Odbc> for bool {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::Bool(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for bool {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::Bool(b) => Ok(*b),
            OdbcValueData::TinyInt(i) => Ok(*i != 0),
            OdbcValueData::Int(i) => Ok(*i != 0),
            OdbcValueData::Text(s) => {
                match s.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => Ok(true),
                    "false" | "0" | "no" | "off" => Ok(false),
                    _ => Err(format!("Cannot decode '{}' as bool", s).into()),
                }
            }
            _ => Err("Cannot decode value as bool".into()),
        }
    }
}

// ============================================================================
// Integers
// ============================================================================

impl Type<Odbc> for i8 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::tiny_int()
    }
}

impl<'q> Encode<'q, Odbc> for i8 {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::TinyInt(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for i8 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::TinyInt(i) => Ok(*i),
            OdbcValueData::SmallInt(i) => Ok(*i as i8),
            OdbcValueData::Int(i) => Ok(*i as i8),
            OdbcValueData::BigInt(i) => Ok(*i as i8),
            OdbcValueData::Text(s) => s.parse().map_err(|e| Box::new(e) as BoxDynError),
            _ => Err("Cannot decode value as i8".into()),
        }
    }
}

impl Type<Odbc> for i16 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::small_int()
    }
}

impl<'q> Encode<'q, Odbc> for i16 {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::SmallInt(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for i16 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::TinyInt(i) => Ok(*i as i16),
            OdbcValueData::SmallInt(i) => Ok(*i),
            OdbcValueData::Int(i) => Ok(*i as i16),
            OdbcValueData::BigInt(i) => Ok(*i as i16),
            OdbcValueData::Text(s) => s.parse().map_err(|e| Box::new(e) as BoxDynError),
            _ => Err("Cannot decode value as i16".into()),
        }
    }
}

impl Type<Odbc> for i32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::integer()
    }
}

impl<'q> Encode<'q, Odbc> for i32 {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::Int(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for i32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::TinyInt(i) => Ok(*i as i32),
            OdbcValueData::SmallInt(i) => Ok(*i as i32),
            OdbcValueData::Int(i) => Ok(*i),
            OdbcValueData::BigInt(i) => Ok(*i as i32),
            OdbcValueData::Text(s) => s.parse().map_err(|e| Box::new(e) as BoxDynError),
            _ => Err("Cannot decode value as i32".into()),
        }
    }
}

impl Type<Odbc> for i64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::big_int()
    }
}

impl<'q> Encode<'q, Odbc> for i64 {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::BigInt(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for i64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::TinyInt(i) => Ok(*i as i64),
            OdbcValueData::SmallInt(i) => Ok(*i as i64),
            OdbcValueData::Int(i) => Ok(*i as i64),
            OdbcValueData::BigInt(i) => Ok(*i),
            OdbcValueData::Text(s) => s.parse().map_err(|e| Box::new(e) as BoxDynError),
            _ => Err("Cannot decode value as i64".into()),
        }
    }
}

// ============================================================================
// Floating Point
// ============================================================================

impl Type<Odbc> for f32 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::real()
    }
}

impl<'q> Encode<'q, Odbc> for f32 {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::Float(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for f32 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::Float(f) => Ok(*f),
            OdbcValueData::Double(f) => Ok(*f as f32),
            OdbcValueData::Int(i) => Ok(*i as f32),
            OdbcValueData::BigInt(i) => Ok(*i as f32),
            OdbcValueData::Text(s) => s.parse().map_err(|e| Box::new(e) as BoxDynError),
            _ => Err("Cannot decode value as f32".into()),
        }
    }
}

impl Type<Odbc> for f64 {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::double()
    }
}

impl<'q> Encode<'q, Odbc> for f64 {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::Double(*self));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for f64 {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::Float(f) => Ok(*f as f64),
            OdbcValueData::Double(f) => Ok(*f),
            OdbcValueData::Int(i) => Ok(*i as f64),
            OdbcValueData::BigInt(i) => Ok(*i as f64),
            OdbcValueData::Text(s) => s.parse().map_err(|e| Box::new(e) as BoxDynError),
            _ => Err("Cannot decode value as f64".into()),
        }
    }
}

// ============================================================================
// String
// ============================================================================

impl Type<Odbc> for String {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varchar(255)
    }

    fn compatible(ty: &OdbcTypeInfo) -> bool {
        use odbc_api::DataType;
        matches!(
            ty.data_type(),
            DataType::Char { .. }
                | DataType::Varchar { .. }
                | DataType::LongVarchar { .. }
                | DataType::WChar { .. }
                | DataType::WVarchar { .. }
                | DataType::WLongVarchar { .. }
        )
    }
}

impl<'q> Encode<'q, Odbc> for String {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::Text(std::borrow::Cow::Owned(self.clone())));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for String {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::Text(s) => Ok(s.clone()),
            OdbcValueData::Int(i) => Ok(i.to_string()),
            OdbcValueData::BigInt(i) => Ok(i.to_string()),
            OdbcValueData::Float(f) => Ok(f.to_string()),
            OdbcValueData::Double(f) => Ok(f.to_string()),
            OdbcValueData::Bool(b) => Ok(b.to_string()),
            _ => Err("Cannot decode value as String".into()),
        }
    }
}

impl Type<Odbc> for str {
    fn type_info() -> OdbcTypeInfo {
        <String as Type<Odbc>>::type_info()
    }

    fn compatible(ty: &OdbcTypeInfo) -> bool {
        <String as Type<Odbc>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Odbc> for &'q str {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::Text(std::borrow::Cow::Borrowed(*self)));
        Ok(IsNull::No)
    }
}

// ============================================================================
// Bytes
// ============================================================================

impl Type<Odbc> for Vec<u8> {
    fn type_info() -> OdbcTypeInfo {
        OdbcTypeInfo::varbinary(255)
    }

    fn compatible(ty: &OdbcTypeInfo) -> bool {
        use odbc_api::DataType;
        matches!(
            ty.data_type(),
            DataType::Binary { .. }
                | DataType::Varbinary { .. }
                | DataType::LongVarbinary { .. }
        )
    }
}

impl<'q> Encode<'q, Odbc> for Vec<u8> {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::Binary(std::borrow::Cow::Owned(self.clone())));
        Ok(IsNull::No)
    }
}

impl<'r> Decode<'r, Odbc> for Vec<u8> {
    fn decode(value: OdbcValueRef<'r>) -> Result<Self, BoxDynError> {
        match value.data {
            OdbcValueData::Binary(b) => Ok(b.clone()),
            OdbcValueData::Text(s) => Ok(s.as_bytes().to_vec()),
            _ => Err("Cannot decode value as Vec<u8>".into()),
        }
    }
}

impl Type<Odbc> for [u8] {
    fn type_info() -> OdbcTypeInfo {
        <Vec<u8> as Type<Odbc>>::type_info()
    }

    fn compatible(ty: &OdbcTypeInfo) -> bool {
        <Vec<u8> as Type<Odbc>>::compatible(ty)
    }
}

impl<'q> Encode<'q, Odbc> for &'q [u8] {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<OdbcArgumentValue<'q>>,
    ) -> Result<IsNull, BoxDynError> {
        buf.push(OdbcArgumentValue::Binary(std::borrow::Cow::Borrowed(*self)));
        Ok(IsNull::No)
    }
}

// Note: Option<T> implementations for Type, Encode, Decode are provided by sqlx_core
