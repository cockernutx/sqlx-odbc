//! ODBC connection implementation.

use crate::odbc::{Odbc, OdbcConnectOptions, OdbcQueryResult};
use odbc_api::SharedConnection;
use sqlx_core::connection::Connection;
use sqlx_core::error::Error;
use sqlx_core::transaction::Transaction;
use std::future::Future;
use std::pin::Pin;

mod executor;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// A connection to an ODBC-accessible database.
///
/// ODBC uses a blocking C API, so we offload blocking calls to the runtime's blocking
/// thread-pool via `spawn_blocking` and synchronize access with a mutex.
pub struct OdbcConnection {
    /// The underlying ODBC connection (wrapped for thread safety via SharedConnection)
    pub(crate) conn: SharedConnection<'static>,
    /// Connection options
    pub(crate) options: OdbcConnectOptions,
    /// Current transaction depth
    pub(crate) transaction_depth: usize,
    /// Whether a rollback is needed
    pub(crate) needs_rollback: bool,
}

// SAFETY: OdbcConnection uses SharedConnection which wraps the connection in Arc<Mutex<>>
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
    /// Execute a blocking operation on the connection
    pub(crate) async fn with_conn<R, F, S>(&mut self, operation: S, f: F) -> Result<R, Error>
    where
        R: Send + 'static,
        F: FnOnce(&mut odbc_api::Connection<'static>) -> Result<R, Error> + Send + 'static,
        S: std::fmt::Display + Send + 'static,
    {
        let conn = self.conn.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn_guard = conn.lock().map_err(|_| {
                Error::Protocol(format!("ODBC {}: failed to lock connection", operation))
            })?;
            f(&mut conn_guard)
        })
        .await
        .map_err(|_| Error::WorkerCrashed)?
    }

    /// Establish a new connection with the given options
    pub async fn establish(options: &OdbcConnectOptions) -> Result<Self, Error> {
        let conn_string = options.connection_string.clone();
        let options = options.clone();

        let shared_conn = tokio::task::spawn_blocking(move || {
            // Get the global ODBC environment
            let env = odbc_api::environment()
                .map_err(|e| Error::Configuration(e.to_string().into()))?;
            
            // Create the actual connection
            let conn = env
                .connect_with_connection_string(&conn_string, odbc_api::ConnectionOptions::default())
                .map_err(|e| Error::Configuration(e.to_string().into()))?;
            
            // Wrap in SharedConnection for thread-safe access
            let shared_conn = odbc_api::SharedConnection::new(std::sync::Mutex::new(conn));
            Ok::<_, Error>(shared_conn)
        })
        .await
        .map_err(|_| Error::WorkerCrashed)??;

        Ok(Self {
            conn: shared_conn,
            options,
            transaction_depth: 0,
            needs_rollback: false,
        })
    }

    /// Execute a raw SQL statement without returning results
    pub async fn execute_raw(&mut self, sql: &str) -> Result<OdbcQueryResult, Error> {
        let sql = sql.to_string();

        self.with_conn("execute_raw", move |conn| {
            // Execute the statement
            match conn.execute(&sql, (), None) {
                Ok(Some(cursor)) => {
                    // For statements that return results, we don't count rows here
                    drop(cursor);
                    Ok(OdbcQueryResult::new(0))
                }
                Ok(None) => Ok(OdbcQueryResult::new(0)),
                Err(e) => Err(Error::Protocol(e.to_string())),
            }
        })
        .await
    }

    /// Begin a transaction by disabling autocommit
    pub(crate) async fn begin_blocking(&mut self) -> Result<(), Error> {
        self.with_conn("begin", move |conn| {
            conn.set_autocommit(false)
                .map_err(|e| Error::Protocol(e.to_string()))?;
            Ok(())
        })
        .await
    }

    /// Commit the current transaction
    pub(crate) async fn commit_blocking(&mut self) -> Result<(), Error> {
        self.with_conn("commit", move |conn| {
            conn.commit()
                .map_err(|e| Error::Protocol(e.to_string()))?;
            conn.set_autocommit(true)
                .map_err(|e| Error::Protocol(e.to_string()))?;
            Ok(())
        })
        .await
    }

    /// Rollback the current transaction
    pub(crate) async fn rollback_blocking(&mut self) -> Result<(), Error> {
        self.with_conn("rollback", move |conn| {
            conn.rollback()
                .map_err(|e| Error::Protocol(e.to_string()))?;
            conn.set_autocommit(true)
                .map_err(|e| Error::Protocol(e.to_string()))?;
            Ok(())
        })
        .await
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
