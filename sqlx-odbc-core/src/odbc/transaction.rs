//! ODBC transaction manager.

use crate::odbc::{Odbc, OdbcConnection};
use sqlx_core::error::Error;
use sqlx_core::transaction::TransactionManager;
use std::borrow::Cow;
use std::future::Future;
use std::pin::Pin;

/// Transaction manager for ODBC connections.
pub struct OdbcTransactionManager;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

impl TransactionManager for OdbcTransactionManager {
    type Database = Odbc;

    fn begin<'c>(conn: &'c mut OdbcConnection, _statement: Option<Cow<'static, str>>) -> BoxFuture<'c, Result<(), Error>> {
        Box::pin(async move {
            // Use ODBC's native autocommit control for reliable transaction management
            conn.begin_blocking().await?;
            conn.transaction_depth += 1;
            Ok(())
        })
    }

    fn commit(conn: &mut OdbcConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if conn.transaction_depth > 0 {
                conn.commit_blocking().await?;
                conn.transaction_depth -= 1;
            }
            Ok(())
        })
    }

    fn rollback(conn: &mut OdbcConnection) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            if conn.transaction_depth > 0 {
                conn.rollback_blocking().await?;
                conn.transaction_depth -= 1;
            }
            Ok(())
        })
    }

    fn start_rollback(conn: &mut OdbcConnection) {
        // Mark that we need to rollback - actual rollback happens in rollback()
        conn.needs_rollback = true;
    }

    fn get_transaction_depth(conn: &OdbcConnection) -> usize {
        conn.transaction_depth
    }
}
