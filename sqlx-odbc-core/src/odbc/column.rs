//! ODBC column definition.

use crate::odbc::{Odbc, OdbcTypeInfo};
use sqlx_core::column::{Column, ColumnIndex};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::Error;

/// A column from an ODBC result set.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "offline", derive(serde::Serialize, serde::Deserialize))]
pub struct OdbcColumn {
    pub(crate) ordinal: usize,
    pub(crate) name: UStr,
    pub(crate) type_info: OdbcTypeInfo,
}

impl OdbcColumn {
    /// Create a new column
    pub fn new(ordinal: usize, name: impl Into<UStr>, type_info: OdbcTypeInfo) -> Self {
        Self {
            ordinal,
            name: name.into(),
            type_info,
        }
    }
}

impl Column for OdbcColumn {
    type Database = Odbc;

    fn ordinal(&self) -> usize {
        self.ordinal
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn type_info(&self) -> &OdbcTypeInfo {
        &self.type_info
    }
}

// Implement ColumnIndex for usize (index-based access)
impl ColumnIndex<crate::odbc::OdbcRow> for usize {
    fn index(&self, row: &crate::odbc::OdbcRow) -> Result<usize, Error> {
        if *self < row.columns.len() {
            Ok(*self)
        } else {
            Err(Error::ColumnIndexOutOfBounds {
                index: *self,
                len: row.columns.len(),
            })
        }
    }
}

// Implement ColumnIndex for &str (name-based access)
impl ColumnIndex<crate::odbc::OdbcRow> for &str {
    fn index(&self, row: &crate::odbc::OdbcRow) -> Result<usize, Error> {
        row.columns
            .iter()
            .position(|col| &*col.name == *self)
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
    }
}
