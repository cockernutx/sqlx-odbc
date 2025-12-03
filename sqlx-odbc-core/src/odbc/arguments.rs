//! ODBC query arguments.

use crate::odbc::database::OdbcArgumentValue;
use crate::odbc::Odbc;
use sqlx_core::arguments::Arguments;
use sqlx_core::encode::Encode;
use sqlx_core::error::BoxDynError;
use sqlx_core::types::Type;
use std::fmt::{self, Write};

/// Arguments for an ODBC query.
#[derive(Debug, Default, Clone)]
pub struct OdbcArguments<'q> {
    pub(crate) values: Vec<OdbcArgumentValue<'q>>,
}

impl<'q> OdbcArguments<'q> {
    /// Create a new empty arguments container
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Create with a specific capacity
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
        }
    }

    /// Add a value to the arguments
    pub fn add<T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: 'q + Encode<'q, Odbc> + Type<Odbc>,
    {
        let _ = value.encode(&mut self.values)?;
        Ok(())
    }

    /// Get the values as a slice
    pub fn values(&self) -> &[OdbcArgumentValue<'q>] {
        &self.values
    }

    /// Get the number of arguments
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if arguments are empty
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl<'q> Arguments<'q> for OdbcArguments<'q> {
    type Database = Odbc;

    fn reserve(&mut self, additional: usize, _size_hint: usize) {
        self.values.reserve(additional);
    }

    fn add<T>(&mut self, value: T) -> Result<(), BoxDynError>
    where
        T: 'q + Encode<'q, Self::Database> + Type<Self::Database>,
    {
        let _ = value.encode(&mut self.values)?;
        Ok(())
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn format_placeholder<W: Write>(&self, writer: &mut W) -> fmt::Result {
        // ODBC uses ? as placeholder
        writer.write_char('?')
    }
}

// Implement IntoArguments for OdbcArguments
impl<'q> sqlx_core::arguments::IntoArguments<'q, Odbc> for OdbcArguments<'q> {
    fn into_arguments(self) -> OdbcArguments<'q> {
        self
    }
}
