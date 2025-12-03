//! ODBC prepared statement.

use crate::odbc::{Odbc, OdbcColumn, OdbcTypeInfo};
use sqlx_core::ext::ustr::UStr;
use sqlx_core::statement::Statement;
use sqlx_core::Either;
use sqlx_core::Error;
use sqlx_core::HashMap;
use std::borrow::Cow;
use std::sync::Arc;

/// Metadata for a prepared ODBC statement.
#[derive(Debug, Default, Clone)]
pub struct OdbcStatementMetadata {
    pub(crate) columns: Vec<OdbcColumn>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,
    pub(crate) parameters: Vec<OdbcTypeInfo>,
}

impl OdbcStatementMetadata {
    /// Create new statement metadata
    pub fn new(columns: Vec<OdbcColumn>, parameters: Vec<OdbcTypeInfo>) -> Self {
        let column_names = columns
            .iter()
            .enumerate()
            .map(|(i, col)| (col.name.clone(), i))
            .collect();

        Self {
            columns,
            column_names: Arc::new(column_names),
            parameters,
        }
    }
}

/// A prepared ODBC statement.
#[derive(Debug, Clone)]
pub struct OdbcStatement<'q> {
    pub(crate) sql: Cow<'q, str>,
    pub(crate) metadata: Arc<OdbcStatementMetadata>,
}

impl<'q> OdbcStatement<'q> {
    /// Create a new statement with the given SQL
    pub fn new(sql: impl Into<Cow<'q, str>>) -> Self {
        Self {
            sql: sql.into(),
            metadata: Arc::new(OdbcStatementMetadata::default()),
        }
    }

    /// Create a new statement with SQL and metadata
    pub fn with_metadata(sql: impl Into<Cow<'q, str>>, metadata: OdbcStatementMetadata) -> Self {
        Self {
            sql: sql.into(),
            metadata: Arc::new(metadata),
        }
    }
}

impl<'q> Statement<'q> for OdbcStatement<'q> {
    type Database = Odbc;

    fn to_owned(&self) -> OdbcStatement<'static> {
        OdbcStatement {
            sql: Cow::Owned(self.sql.clone().into_owned()),
            metadata: self.metadata.clone(),
        }
    }

    fn sql(&self) -> &str {
        &self.sql
    }

    fn parameters(&self) -> Option<Either<&[OdbcTypeInfo], usize>> {
        if self.metadata.parameters.is_empty() {
            None
        } else {
            Some(Either::Left(&self.metadata.parameters))
        }
    }

    fn columns(&self) -> &[OdbcColumn] {
        &self.metadata.columns
    }

    sqlx_core::impl_statement_query!(crate::odbc::OdbcArguments<'_>);
}

// Column index by name for statement
impl sqlx_core::column::ColumnIndex<OdbcStatement<'_>> for &str {
    fn index(&self, statement: &OdbcStatement<'_>) -> Result<usize, Error> {
        statement
            .metadata
            .column_names
            .get(*self)
            .copied()
            .ok_or_else(|| Error::ColumnNotFound((*self).into()))
    }
}
