//! ODBC type information.

use odbc_api::DataType;
use sqlx_core::type_info::TypeInfo;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// Type information for an ODBC type.
#[derive(Debug, Clone, Eq)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct OdbcTypeInfo {
    #[cfg_attr(feature = "offline", serde(skip))]
    pub(crate) data_type: DataType,
}

impl OdbcTypeInfo {
    /// Create a new OdbcTypeInfo with the given data type
    pub const fn new(data_type: DataType) -> Self {
        Self { data_type }
    }

    /// Get the underlying data type
    pub const fn data_type(&self) -> DataType {
        self.data_type
    }

    // Common type constants
    pub const fn null() -> Self {
        Self { data_type: DataType::Unknown }
    }

    pub const fn bit() -> Self {
        Self { data_type: DataType::Bit }
    }

    pub const fn tiny_int() -> Self {
        Self { data_type: DataType::TinyInt }
    }

    pub const fn small_int() -> Self {
        Self { data_type: DataType::SmallInt }
    }

    pub const fn integer() -> Self {
        Self { data_type: DataType::Integer }
    }

    pub const fn big_int() -> Self {
        Self { data_type: DataType::BigInt }
    }

    pub const fn real() -> Self {
        Self { data_type: DataType::Real }
    }

    pub const fn double() -> Self {
        Self { data_type: DataType::Double }
    }

    pub const fn date() -> Self {
        Self { data_type: DataType::Date }
    }

    pub fn varchar(length: usize) -> Self {
        Self {
            data_type: DataType::Varchar { length: std::num::NonZero::new(length) },
        }
    }

    pub fn varbinary(length: usize) -> Self {
        Self {
            data_type: DataType::Varbinary { length: std::num::NonZero::new(length) },
        }
    }
}

impl TypeInfo for OdbcTypeInfo {
    fn is_null(&self) -> bool {
        matches!(self.data_type, DataType::Unknown)
    }

    fn name(&self) -> &str {
        data_type_name(self.data_type)
    }
}

impl Display for OdbcTypeInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.pad(self.name())
    }
}

impl PartialEq<OdbcTypeInfo> for OdbcTypeInfo {
    fn eq(&self, other: &OdbcTypeInfo) -> bool {
        // Compare by type category, not exact parameters
        std::mem::discriminant(&self.data_type) == std::mem::discriminant(&other.data_type)
    }
}

/// Get the display name for a DataType
pub fn data_type_name(dt: DataType) -> &'static str {
    match dt {
        DataType::BigInt => "BIGINT",
        DataType::Binary { .. } => "BINARY",
        DataType::Bit => "BIT",
        DataType::Char { .. } => "CHAR",
        DataType::Date => "DATE",
        DataType::Decimal { .. } => "DECIMAL",
        DataType::Double => "DOUBLE",
        DataType::Float { .. } => "FLOAT",
        DataType::Integer => "INTEGER",
        DataType::LongVarbinary { .. } => "LONGVARBINARY",
        DataType::LongVarchar { .. } => "LONGVARCHAR",
        DataType::Numeric { .. } => "NUMERIC",
        DataType::Real => "REAL",
        DataType::SmallInt => "SMALLINT",
        DataType::Time { .. } => "TIME",
        DataType::Timestamp { .. } => "TIMESTAMP",
        DataType::TinyInt => "TINYINT",
        DataType::Varbinary { .. } => "VARBINARY",
        DataType::Varchar { .. } => "VARCHAR",
        DataType::WChar { .. } => "WCHAR",
        DataType::WLongVarchar { .. } => "WLONGVARCHAR",
        DataType::WVarchar { .. } => "WVARCHAR",
        DataType::Unknown => "UNKNOWN",
        DataType::Other { .. } => "OTHER",
    }
}

/// Extension trait for DataType with helper methods
pub trait DataTypeExt {
    /// Check if this is a character/string type
    fn accepts_character_data(self) -> bool;

    /// Check if this is a binary type
    fn accepts_binary_data(self) -> bool;

    /// Check if this is a numeric type
    fn accepts_numeric_data(self) -> bool;

    /// Check if this is a date/time type
    fn accepts_datetime_data(self) -> bool;
}

impl DataTypeExt for DataType {
    fn accepts_character_data(self) -> bool {
        matches!(
            self,
            DataType::Char { .. }
                | DataType::Varchar { .. }
                | DataType::LongVarchar { .. }
                | DataType::WChar { .. }
                | DataType::WVarchar { .. }
                | DataType::WLongVarchar { .. }
        )
    }

    fn accepts_binary_data(self) -> bool {
        matches!(
            self,
            DataType::Binary { .. } | DataType::Varbinary { .. } | DataType::LongVarbinary { .. }
        )
    }

    fn accepts_numeric_data(self) -> bool {
        matches!(
            self,
            DataType::TinyInt
                | DataType::SmallInt
                | DataType::Integer
                | DataType::BigInt
                | DataType::Real
                | DataType::Float { .. }
                | DataType::Double
                | DataType::Decimal { .. }
                | DataType::Numeric { .. }
        )
    }

    fn accepts_datetime_data(self) -> bool {
        matches!(
            self,
            DataType::Date | DataType::Time { .. } | DataType::Timestamp { .. }
        )
    }
}
