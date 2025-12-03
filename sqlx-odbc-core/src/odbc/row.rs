//! ODBC row definition.

use crate::odbc::{Odbc, OdbcColumn, OdbcValue, OdbcValueRef};
use sqlx_core::column::ColumnIndex;
use sqlx_core::row::Row;
use sqlx_core::Error;

/// A row from an ODBC result set.
#[derive(Debug, Clone)]
pub struct OdbcRow {
    pub(crate) columns: Vec<OdbcColumn>,
    pub(crate) values: Vec<OdbcValue>,
}

impl OdbcRow {
    /// Create a new row with the given columns and values
    pub fn new(columns: Vec<OdbcColumn>, values: Vec<OdbcValue>) -> Self {
        debug_assert_eq!(columns.len(), values.len());
        Self { columns, values }
    }

    /// Get the number of columns in this row
    pub fn len(&self) -> usize {
        self.columns.len()
    }

    /// Check if the row is empty
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Get a value by index
    pub fn get_value(&self, index: usize) -> Option<&OdbcValue> {
        self.values.get(index)
    }

    /// Get a column by index
    pub fn get_column(&self, index: usize) -> Option<&OdbcColumn> {
        self.columns.get(index)
    }
}

impl Row for OdbcRow {
    type Database = Odbc;

    fn columns(&self) -> &[OdbcColumn] {
        &self.columns
    }

    fn try_get_raw<I>(&self, index: I) -> Result<OdbcValueRef<'_>, Error>
    where
        I: ColumnIndex<Self>,
    {
        let index = index.index(self)?;
        let value = &self.values[index];
        let type_info = self.columns[index].type_info.clone();
        Ok(OdbcValueRef::new(&value.data, type_info))
    }
}
