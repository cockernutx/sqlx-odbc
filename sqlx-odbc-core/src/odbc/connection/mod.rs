//! ODBC connection implementation.

use crate::odbc::{Odbc, OdbcConnectOptions, OdbcQueryResult};
use odbc_api::Environment;
use sqlx_core::connection::Connection;
use sqlx_core::error::Error;
use sqlx_core::transaction::Transaction;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, OnceLock};

mod executor;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

// Global ODBC environment (thread-safe singleton)
static ODBC_ENV: OnceLock<Arc<Environment>> = OnceLock::new();

fn get_odbc_environment() -> Result<Arc<Environment>, Error> {
    let env = ODBC_ENV.get_or_init(|| {
        Arc::new(
            Environment::new().expect("Failed to create ODBC environment"),
        )
    });
    Ok(env.clone())
}

/// A connection to an ODBC-accessible database.
pub struct OdbcConnection {
    /// The underlying ODBC connection (wrapped for thread safety)
    pub(crate) inner: Arc<Mutex<OdbcConnectionInner>>,
    /// Connection options
    pub(crate) options: OdbcConnectOptions,
    /// Current transaction depth
    pub(crate) transaction_depth: usize,
    /// Whether a rollback is needed
    pub(crate) needs_rollback: bool,
}

/// Inner connection state that holds the actual ODBC connection
pub(crate) struct OdbcConnectionInner {
    // We store the connection string and recreate connections as needed
    // This is simpler than trying to manage ODBC connection lifetimes
    pub(crate) connection_string: String,
    pub(crate) env: Arc<Environment>,
}

// SAFETY: OdbcConnection uses Arc<Mutex<>> for thread-safe access
unsafe impl Send for OdbcConnection {}
unsafe impl Sync for OdbcConnection {}

impl std::fmt::Debug for OdbcConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OdbcConnection")
            .field("transaction_depth", &self.transaction_depth)
            .field("needs_rollback", &self.needs_rollback)
            .finish()
    }
}

impl OdbcConnection {
    /// Establish a new connection with the given options
    pub async fn establish(options: &OdbcConnectOptions) -> Result<Self, Error> {
        let env = get_odbc_environment()?;
        let conn_string = options.connection_string.clone();
        let options = options.clone();

        // Test the connection by actually connecting
        let env_clone = env.clone();
        let conn_string_clone = conn_string.clone();
        
        tokio::task::spawn_blocking(move || {
            // Test connection and drop it immediately
            let _conn = env_clone
                .connect_with_connection_string(&conn_string_clone, odbc_api::ConnectionOptions::default())
                .map_err(|e| Error::Configuration(e.to_string().into()))?;
            Ok::<(), Error>(())
        })
        .await
        .map_err(|_| Error::WorkerCrashed)??;

        let inner = OdbcConnectionInner {
            connection_string: conn_string,
            env,
        };

        Ok(Self {
            inner: Arc::new(Mutex::new(inner)),
            options,
            transaction_depth: 0,
            needs_rollback: false,
        })
    }

    /// Execute a raw SQL statement without returning results
    pub async fn execute_raw(&mut self, sql: &str) -> Result<OdbcQueryResult, Error> {
        let inner = self.inner.clone();
        let sql = sql.to_string();

        let rows_affected = tokio::task::spawn_blocking(move || {
            let inner = inner.lock().map_err(|_| {
                Error::Protocol("Failed to lock ODBC connection".into())
            })?;

            // Create a new connection for this operation
            let conn = inner.env
                .connect_with_connection_string(&inner.connection_string, odbc_api::ConnectionOptions::default())
                .map_err(|e| Error::Protocol(e.to_string()))?;

            // Execute the statement
            match conn.execute(&sql, (), None) {
                Ok(Some(cursor)) => {
                    // For statements that return results, we don't count rows here
                    drop(cursor);
                    Ok(0u64)
                }
                Ok(None) => Ok(0),
                Err(e) => Err(Error::Protocol(e.to_string())),
            }
        })
        .await
        .map_err(|_| Error::WorkerCrashed)??;

        Ok(OdbcQueryResult::new(rows_affected))
    }

    /// Get the connection options
    pub fn options(&self) -> &OdbcConnectOptions {
        &self.options
    }
}

impl Connection for OdbcConnection {
    type Database = Odbc;
    type Options = OdbcConnectOptions;

    fn close(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            // Connection will be closed when dropped
            drop(self);
            Ok(())
        })
    }

    fn close_hard(self) -> BoxFuture<'static, Result<(), Error>> {
        Box::pin(async move {
            drop(self);
            Ok(())
        })
    }

    fn ping(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            // Execute a simple query to check connection health
            self.execute_raw("SELECT 1").await?;
            Ok(())
        })
    }

    fn begin(
        &mut self,
    ) -> BoxFuture<'_, Result<Transaction<'_, Self::Database>, Error>>
    where
        Self: Sized,
    {
        Transaction::begin(self, None)
    }

    fn shrink_buffers(&mut self) {
        // No-op for ODBC
    }

    fn flush(&mut self) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async { Ok(()) })
    }

    fn should_flush(&self) -> bool {
        false
    }
}
