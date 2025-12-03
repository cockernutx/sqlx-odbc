//! ODBC error types.

use odbc_api::Error as OdbcApiError;
use sqlx_core::error::{BoxDynError, DatabaseError, ErrorKind};
use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// An error returned from an ODBC database.
#[derive(Debug)]
pub struct OdbcDatabaseError {
    pub(crate) inner: OdbcApiError,
    pub(crate) message: String,
    pub(crate) sqlstate: Option<String>,
}

impl OdbcDatabaseError {
    /// Create a new OdbcDatabaseError from an odbc_api::Error
    pub fn new(error: OdbcApiError) -> Self {
        let message = error.to_string();
        let sqlstate = extract_sqlstate(&error);
        Self {
            inner: error,
            message,
            sqlstate,
        }
    }

    /// Get the SQLSTATE code if available
    pub fn sqlstate(&self) -> Option<&str> {
        self.sqlstate.as_deref()
    }
}

impl Display for OdbcDatabaseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        write!(f, "{}", self.message)
    }
}

impl StdError for OdbcDatabaseError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.inner)
    }
}

impl DatabaseError for OdbcDatabaseError {
    fn message(&self) -> &str {
        &self.message
    }

    fn code(&self) -> Option<Cow<'_, str>> {
        self.sqlstate.as_ref().map(|s| Cow::Borrowed(s.as_str()))
    }

    fn as_error(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self
    }

    fn as_error_mut(&mut self) -> &mut (dyn StdError + Send + Sync + 'static) {
        self
    }

    fn into_error(self: Box<Self>) -> BoxDynError {
        self
    }

    fn kind(&self) -> ErrorKind {
        // Try to determine the error kind from SQLSTATE
        match self.sqlstate.as_deref() {
            // Integrity constraint violations
            Some(s) if s.starts_with("23") => {
                match s {
                    "23505" => ErrorKind::UniqueViolation,
                    "23503" => ErrorKind::ForeignKeyViolation,
                    "23514" => ErrorKind::CheckViolation,
                    "23502" => ErrorKind::NotNullViolation,
                    _ => ErrorKind::Other
                }
            }
            _ => ErrorKind::Other,
        }
    }

    fn is_transient_in_connect_phase(&self) -> bool {
        // Connection-related SQLSTATE codes that might be transient
        match self.sqlstate.as_deref() {
            Some(s) if s.starts_with("08") => true, // Connection errors
            Some("HYT00") => true, // Timeout
            Some("HYT01") => true, // Connection timeout
            _ => false,
        }
    }
}

impl From<OdbcApiError> for OdbcDatabaseError {
    fn from(error: OdbcApiError) -> Self {
        Self::new(error)
    }
}

// Note: We cannot implement From<OdbcApiError> for sqlx_core::Error due to orphan rules.
// Use OdbcDatabaseError::new(error).into() or Error::Database(Box::new(OdbcDatabaseError::new(error)))

/// Extract SQLSTATE from an ODBC error if available
fn extract_sqlstate(error: &OdbcApiError) -> Option<String> {
    // Try to extract SQLSTATE from the error message or structure
    // ODBC errors typically include SQLSTATE in format [SQLSTATE]
    let msg = error.to_string();
    
    // Look for pattern like [HY000] or [23505]
    if let Some(start) = msg.find('[') {
        if let Some(end) = msg[start..].find(']') {
            let state = &msg[start + 1..start + end];
            if state.len() == 5 && state.chars().all(|c| c.is_ascii_alphanumeric()) {
                return Some(state.to_string());
            }
        }
    }
    
    None
}
