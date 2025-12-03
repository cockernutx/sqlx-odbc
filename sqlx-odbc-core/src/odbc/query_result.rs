//! ODBC query result.

/// Result of an ODBC query execution.
#[derive(Debug, Clone, Default)]
pub struct OdbcQueryResult {
    pub(crate) rows_affected: u64,
}

impl OdbcQueryResult {
    /// Create a new query result
    pub fn new(rows_affected: u64) -> Self {
        Self { rows_affected }
    }

    /// Get the number of rows affected by the query
    pub fn rows_affected(&self) -> u64 {
        self.rows_affected
    }
}

impl Extend<OdbcQueryResult> for OdbcQueryResult {
    fn extend<T: IntoIterator<Item = OdbcQueryResult>>(&mut self, iter: T) {
        for result in iter {
            self.rows_affected += result.rows_affected;
        }
    }
}
