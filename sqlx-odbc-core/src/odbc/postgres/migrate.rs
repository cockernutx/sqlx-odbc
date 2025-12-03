//! Migration support for PostgreSQL via ODBC.
//!
//! This module provides PostgreSQL-specific migration functionality through extension traits.
//! Users must explicitly import these traits to enable migration support.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use sqlx_odbc::odbc::postgres::{PostgresMigrateExt, PostgresMigrateDatabaseExt};
//! use sqlx_odbc::odbc::{Odbc, OdbcConnection, OdbcConnectOptions};
//! use sqlx_core::connection::ConnectOptions;
//!
//! // Create a database
//! Odbc::postgres_create_database("Driver={PostgreSQL};Server=localhost;Database=mydb").await?;
//!
//! // Run migrations
//! let migrator = Migrator::new(std::path::Path::new("./migrations")).await?;
//! connection.postgres_run_migrations(&migrator).await?;
//! ```

use std::time::Duration;
use std::time::Instant;

use futures_core::future::BoxFuture;
use odbc_api::Cursor;

use sqlx_core::migrate::MigrateError;
use sqlx_core::migrate::{AppliedMigration, Migration, Migrator};

use crate::odbc::connection::OdbcConnectionInner;
use crate::odbc::{Odbc, OdbcConnectOptions, OdbcConnection};
use sqlx_core::connection::ConnectOptions;
use sqlx_core::error::Error;
use std::sync::{Arc, Mutex};

/// Default migrations table name
const MIGRATIONS_TABLE: &str = "_sqlx_migrations";

/// Extension trait for PostgreSQL database management operations.
///
/// This trait provides PostgreSQL-specific database creation and management
/// functionality. Import this trait to enable these methods on `Odbc`.
///
/// # Example
///
/// ```rust,ignore
/// use sqlx_odbc::odbc::postgres::PostgresMigrateDatabaseExt;
/// use sqlx_odbc::odbc::Odbc;
///
/// // Create a database
/// Odbc::postgres_create_database("Driver={PostgreSQL};Server=localhost;Database=mydb").await?;
///
/// // Check if database exists
/// let exists = Odbc::postgres_database_exists("...connection_string...").await?;
///
/// // Drop a database
/// Odbc::postgres_drop_database("...connection_string...").await?;
/// ```
pub trait PostgresMigrateDatabaseExt {
    /// Create a PostgreSQL database if it doesn't exist.
    ///
    /// The connection string should include the target database name.
    /// This method connects to the `postgres` maintenance database to perform the creation.
    fn postgres_create_database(url: &str) -> BoxFuture<'_, Result<(), Error>>;

    /// Check if a PostgreSQL database exists.
    fn postgres_database_exists(url: &str) -> BoxFuture<'_, Result<bool, Error>>;

    /// Drop a PostgreSQL database if it exists.
    fn postgres_drop_database(url: &str) -> BoxFuture<'_, Result<(), Error>>;

    /// Force drop a PostgreSQL database, terminating all active connections.
    ///
    /// This terminates all connections to the database before dropping it.
    fn postgres_force_drop_database(url: &str) -> BoxFuture<'_, Result<(), Error>>;
}

/// Extension trait for PostgreSQL migration operations on connections.
///
/// This trait provides PostgreSQL-specific migration functionality.
/// Import this trait to enable migration methods on `OdbcConnection`.
///
/// # Example
///
/// ```rust,ignore
/// use sqlx_odbc::odbc::postgres::PostgresMigrateExt;
/// use sqlx_core::migrate::Migrator;
///
/// let migrator = Migrator::new(std::path::Path::new("./migrations")).await?;
///
/// // Run all pending migrations
/// connection.postgres_run_migrations(&migrator).await?;
///
/// // Or run individual migration operations
/// connection.postgres_ensure_migrations_table().await?;
/// let applied = connection.postgres_list_applied_migrations().await?;
/// ```
pub trait PostgresMigrateExt {
    /// Ensure the migrations table exists, creating it if necessary.
    fn postgres_ensure_migrations_table(&mut self) -> BoxFuture<'_, Result<(), MigrateError>>;

    /// Get the version of a dirty (failed) migration, if any.
    fn postgres_dirty_version(&mut self) -> BoxFuture<'_, Result<Option<i64>, MigrateError>>;

    /// List all applied migrations.
    fn postgres_list_applied_migrations(&mut self) -> BoxFuture<'_, Result<Vec<AppliedMigration>, MigrateError>>;

    /// Acquire an advisory lock for migrations.
    fn postgres_lock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>>;

    /// Release the advisory lock for migrations.
    fn postgres_unlock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>>;

    /// Apply a single migration.
    fn postgres_apply<'e, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<Duration, MigrateError>>
    where
        'e: 'm;

    /// Revert a single migration.
    fn postgres_revert<'e, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<Duration, MigrateError>>
    where
        'e: 'm;

    /// Run all pending migrations from the migrator.
    ///
    /// This is a convenience method that:
    /// 1. Acquires an advisory lock
    /// 2. Ensures the migrations table exists
    /// 3. Applies all pending migrations
    /// 4. Releases the lock
    fn postgres_run_migrations<'e, 'm>(
        &'e mut self,
        migrator: &'m Migrator,
    ) -> BoxFuture<'m, Result<(), MigrateError>>
    where
        'e: 'm;
}

// ============================================================================
// Implementation for Odbc (database management)
// ============================================================================

impl PostgresMigrateDatabaseExt for Odbc {
    fn postgres_create_database(url: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let conn_string = normalize_connection_string(url);
            let (options, database) = parse_for_maintenance(&conn_string)?;
            let mut conn = options.connect().await?;

            // PostgreSQL syntax - check if exists first, then create
            // We can't use IF NOT EXISTS directly with CREATE DATABASE in PostgreSQL
            let check_sql = format!(
                "SELECT 1 FROM pg_database WHERE datname = '{}'",
                database.replace('\'', "''")
            );
            
            let exists = execute_scalar_bool(&mut conn, &check_sql).await?;
            
            if !exists {
                let sql = format!(
                    "CREATE DATABASE \"{}\"",
                    database.replace('"', "\"\"")
                );
                conn.execute_raw(&sql).await?;
            }
            
            Ok(())
        })
    }

    fn postgres_database_exists(url: &str) -> BoxFuture<'_, Result<bool, Error>> {
        Box::pin(async move {
            let conn_string = normalize_connection_string(url);
            let (options, database) = parse_for_maintenance(&conn_string)?;
            let mut conn = options.connect().await?;

            let sql = format!(
                "SELECT EXISTS(SELECT 1 FROM pg_database WHERE datname = '{}')",
                database.replace('\'', "''")
            );

            execute_scalar_bool(&mut conn, &sql).await
        })
    }

    fn postgres_drop_database(url: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let conn_string = normalize_connection_string(url);
            let (options, database) = parse_for_maintenance(&conn_string)?;
            let mut conn = options.connect().await?;

            let sql = format!(
                "DROP DATABASE IF EXISTS \"{}\"",
                database.replace('"', "\"\"")
            );

            conn.execute_raw(&sql).await?;
            Ok(())
        })
    }

    fn postgres_force_drop_database(url: &str) -> BoxFuture<'_, Result<(), Error>> {
        Box::pin(async move {
            let conn_string = normalize_connection_string(url);
            let (options, database) = parse_for_maintenance(&conn_string)?;
            let mut conn = options.connect().await?;

            // Terminate all connections to the database
            let terminate_sql = format!(
                "SELECT pg_terminate_backend(pg_stat_activity.pid) \
                 FROM pg_stat_activity \
                 WHERE pg_stat_activity.datname = '{}' \
                 AND pid <> pg_backend_pid()",
                database.replace('\'', "''")
            );
            
            let _ = conn.execute_raw(&terminate_sql).await;

            // Now drop the database
            let sql = format!(
                "DROP DATABASE IF EXISTS \"{}\"",
                database.replace('"', "\"\"")
            );

            conn.execute_raw(&sql).await?;
            Ok(())
        })
    }
}

// ============================================================================
// Implementation for OdbcConnection (migration operations)
// ============================================================================

impl PostgresMigrateExt for OdbcConnection {
    fn postgres_ensure_migrations_table(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move {
            let sql = format!(
                r#"
CREATE TABLE IF NOT EXISTS "{table}" (
    version BIGINT PRIMARY KEY,
    description TEXT NOT NULL,
    installed_on TIMESTAMPTZ NOT NULL DEFAULT now(),
    success BOOLEAN NOT NULL,
    checksum BYTEA NOT NULL,
    execution_time BIGINT NOT NULL
);
                "#,
                table = MIGRATIONS_TABLE.replace('"', "\"\"")
            );

            self.execute_raw(&sql).await?;
            Ok(())
        })
    }

    fn postgres_dirty_version(&mut self) -> BoxFuture<'_, Result<Option<i64>, MigrateError>> {
        Box::pin(async move {
            let sql = format!(
                "SELECT version FROM \"{}\" WHERE success = false ORDER BY version LIMIT 1",
                MIGRATIONS_TABLE.replace('"', "\"\"")
            );

            execute_scalar_i64(self, &sql).await
        })
    }

    fn postgres_list_applied_migrations(&mut self) -> BoxFuture<'_, Result<Vec<AppliedMigration>, MigrateError>> {
        Box::pin(async move {
            let sql = format!(
                "SELECT version, checksum FROM \"{}\" ORDER BY version",
                MIGRATIONS_TABLE.replace('"', "\"\"")
            );

            let rows = execute_query_migrations(self, &sql).await?;

            let migrations = rows
                .into_iter()
                .map(|(version, checksum)| AppliedMigration {
                    version,
                    checksum: checksum.into(),
                })
                .collect();

            Ok(migrations)
        })
    }

    fn postgres_lock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move {
            let database_name = current_database(self).await?;
            let lock_id = generate_lock_id(&database_name);

            // PostgreSQL advisory lock
            let sql = format!("SELECT pg_advisory_lock({})", lock_id);

            let _ = self.execute_raw(&sql).await;
            Ok(())
        })
    }

    fn postgres_unlock(&mut self) -> BoxFuture<'_, Result<(), MigrateError>> {
        Box::pin(async move {
            let database_name = current_database(self).await?;
            let lock_id = generate_lock_id(&database_name);

            let sql = format!("SELECT pg_advisory_unlock({})", lock_id);

            let _ = self.execute_raw(&sql).await;
            Ok(())
        })
    }

    fn postgres_apply<'e, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<Duration, MigrateError>>
    where
        'e: 'm,
    {
        Box::pin(async move {
            let start = Instant::now();

            if migration.no_tx {
                execute_migration(self, MIGRATIONS_TABLE, migration).await?;
            } else {
                self.execute_raw("BEGIN")
                    .await
                    .map_err(|e| MigrateError::ExecuteMigration(e, migration.version))?;

                match execute_migration(self, MIGRATIONS_TABLE, migration).await {
                    Ok(()) => {
                        self.execute_raw("COMMIT")
                            .await
                            .map_err(|e| MigrateError::ExecuteMigration(e, migration.version))?;
                    }
                    Err(e) => {
                        let _ = self.execute_raw("ROLLBACK").await;
                        return Err(e);
                    }
                }
            }

            let elapsed = start.elapsed();

            #[allow(clippy::cast_possible_truncation)]
            let sql = format!(
                "UPDATE \"{}\" SET execution_time = {} WHERE version = {}",
                MIGRATIONS_TABLE.replace('"', "\"\""),
                elapsed.as_nanos() as i64,
                migration.version
            );

            let _ = self.execute_raw(&sql).await;
            Ok(elapsed)
        })
    }

    fn postgres_revert<'e, 'm>(
        &'e mut self,
        migration: &'m Migration,
    ) -> BoxFuture<'m, Result<Duration, MigrateError>>
    where
        'e: 'm,
    {
        Box::pin(async move {
            let start = Instant::now();

            if migration.no_tx {
                revert_migration(self, MIGRATIONS_TABLE, migration).await?;
            } else {
                self.execute_raw("BEGIN")
                    .await
                    .map_err(|e| MigrateError::ExecuteMigration(e, migration.version))?;

                match revert_migration(self, MIGRATIONS_TABLE, migration).await {
                    Ok(()) => {
                        self.execute_raw("COMMIT")
                            .await
                            .map_err(|e| MigrateError::ExecuteMigration(e, migration.version))?;
                    }
                    Err(e) => {
                        let _ = self.execute_raw("ROLLBACK").await;
                        return Err(e);
                    }
                }
            }

            Ok(start.elapsed())
        })
    }

    fn postgres_run_migrations<'e, 'm>(
        &'e mut self,
        migrator: &'m Migrator,
    ) -> BoxFuture<'m, Result<(), MigrateError>>
    where
        'e: 'm,
    {
        Box::pin(async move {
            // Acquire lock
            self.postgres_lock().await?;

            let result = async {
                // Ensure migrations table exists
                self.postgres_ensure_migrations_table().await?;

                // Check for dirty migrations
                if let Some(version) = self.postgres_dirty_version().await? {
                    return Err(MigrateError::Dirty(version));
                }

                // Get applied migrations
                let applied: std::collections::HashSet<i64> = self
                    .postgres_list_applied_migrations()
                    .await?
                    .into_iter()
                    .map(|m| m.version)
                    .collect();

                // Apply pending migrations
                for migration in migrator.iter() {
                    if migration.migration_type.is_down_migration() {
                        continue;
                    }

                    if applied.contains(&migration.version) {
                        continue;
                    }

                    log::info!("Applying migration {} - {}", migration.version, migration.description);
                    let elapsed = self.postgres_apply(migration).await?;
                    log::info!(
                        "Applied migration {} in {:?}",
                        migration.version,
                        elapsed
                    );
                }

                Ok(())
            }
            .await;

            // Always release lock
            let _ = self.postgres_unlock().await;

            result
        })
    }
}

// ============================================================================
// Helper functions
// ============================================================================

/// Normalize connection string, handling URL-style prefixes
fn normalize_connection_string(url: &str) -> String {
    if url.starts_with("odbc:") {
        url.strip_prefix("odbc://")
            .or_else(|| url.strip_prefix("odbc:"))
            .unwrap_or(url)
            .to_string()
    } else {
        url.to_string()
    }
}

/// Parse connection string to extract database name and create maintenance connection options.
/// For PostgreSQL, the maintenance database is typically 'postgres' or 'template1'.
fn parse_for_maintenance(connection_string: &str) -> Result<(OdbcConnectOptions, String), Error> {
    let mut database = String::new();
    let mut maintenance_string = String::new();

    for part in connection_string.split(';') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some((key, value)) = part.split_once('=') {
            let key_lower = key.to_lowercase();
            if key_lower == "database" {
                database = value.to_string();
                if !maintenance_string.is_empty() {
                    maintenance_string.push(';');
                }
                // Use 'postgres' unless target is 'postgres', then use 'template1'
                let maintenance_db = if value.to_lowercase() == "postgres" {
                    "template1"
                } else {
                    "postgres"
                };
                maintenance_string.push_str(key);
                maintenance_string.push('=');
                maintenance_string.push_str(maintenance_db);
            } else {
                if !maintenance_string.is_empty() {
                    maintenance_string.push(';');
                }
                maintenance_string.push_str(part);
            }
        } else {
            if !maintenance_string.is_empty() {
                maintenance_string.push(';');
            }
            maintenance_string.push_str(part);
        }
    }

    if database.is_empty() {
        database = "postgres".to_string();
    }

    if !maintenance_string.to_lowercase().contains("database=") {
        if !maintenance_string.is_empty() {
            maintenance_string.push(';');
        }
        maintenance_string.push_str("Database=postgres");
    }

    Ok((OdbcConnectOptions::new(maintenance_string), database))
}

/// Execute a migration SQL and record it in the migrations table
async fn execute_migration(
    conn: &mut OdbcConnection,
    table_name: &str,
    migration: &Migration,
) -> Result<(), MigrateError> {
    conn.execute_raw(migration.sql.as_ref())
        .await
        .map_err(|e| MigrateError::ExecuteMigration(e, migration.version))?;

    // PostgreSQL uses bytea with hex format for binary data
    let checksum_hex = migration
        .checksum
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();

    let sql = format!(
        "INSERT INTO \"{table}\" (version, description, success, checksum, execution_time) \
         VALUES ({version}, '{description}', true, E'\\\\x{checksum}', -1)",
        table = table_name.replace('"', "\"\""),
        version = migration.version,
        description = migration.description.replace('\'', "''"),
        checksum = checksum_hex
    );

    conn.execute_raw(&sql).await?;
    Ok(())
}

/// Revert a migration
async fn revert_migration(
    conn: &mut OdbcConnection,
    table_name: &str,
    migration: &Migration,
) -> Result<(), MigrateError> {
    conn.execute_raw(migration.sql.as_ref())
        .await
        .map_err(|e| MigrateError::ExecuteMigration(e, migration.version))?;

    let sql = format!(
        "DELETE FROM \"{}\" WHERE version = {}",
        table_name.replace('"', "\"\""),
        migration.version
    );

    conn.execute_raw(&sql).await?;
    Ok(())
}

/// Get the current database name
async fn current_database(conn: &mut OdbcConnection) -> Result<String, MigrateError> {
    let inner = conn.inner.clone();

    let result = tokio::task::spawn_blocking(move || {
        execute_scalar_string_sync(inner, "SELECT current_database()".to_string())
    })
    .await
    .map_err(|_| MigrateError::Source(Error::WorkerCrashed.into()))?
    .map_err(|e| MigrateError::Source(e.into()))?;

    Ok(result.unwrap_or_else(|| "postgres".to_string()))
}

/// Generate a lock ID from the database name (same as sqlx-postgres)
fn generate_lock_id(database_name: &str) -> i64 {
    const CRC_IEEE: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    // 0x3d32ad9e chosen by fair dice roll (same as sqlx-postgres)
    0x3d32ad9e * (CRC_IEEE.checksum(database_name.as_bytes()) as i64)
}

/// Execute a query and return a boolean result
async fn execute_scalar_bool(conn: &mut OdbcConnection, sql: &str) -> Result<bool, Error> {
    let inner = conn.inner.clone();
    let sql = sql.to_string();

    tokio::task::spawn_blocking(move || {
        let inner = inner.lock().map_err(|_| {
            Error::Protocol("Failed to lock ODBC connection".into())
        })?;

        let odbc_conn = inner
            .env
            .connect_with_connection_string(&inner.connection_string, odbc_api::ConnectionOptions::default())
            .map_err(|e| Error::Protocol(e.to_string()))?;

        match odbc_conn.execute(&sql, (), None) {
            Ok(Some(mut cursor)) => {
                if let Some(mut row) = cursor.next_row().map_err(|e| Error::Protocol(e.to_string()))? {
                    let mut buf = Vec::new();
                    let has_value = row
                        .get_text(1, &mut buf)
                        .map_err(|e| Error::Protocol(e.to_string()))?;

                    if has_value {
                        let s = String::from_utf8_lossy(&buf);
                        let s = s.trim().to_lowercase();
                        Ok(s == "1" || s == "t" || s == "true")
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            Ok(None) => Ok(false),
            Err(e) => Err(Error::Protocol(e.to_string())),
        }
    })
    .await
    .map_err(|_| Error::WorkerCrashed)?
}

/// Execute a query and return an optional i64 result
async fn execute_scalar_i64(conn: &mut OdbcConnection, sql: &str) -> Result<Option<i64>, MigrateError> {
    let inner = conn.inner.clone();
    let sql = sql.to_string();

    let result = tokio::task::spawn_blocking(move || {
        let inner = inner.lock().map_err(|_| {
            Error::Protocol("Failed to lock ODBC connection".into())
        })?;

        let odbc_conn = inner
            .env
            .connect_with_connection_string(&inner.connection_string, odbc_api::ConnectionOptions::default())
            .map_err(|e| Error::Protocol(e.to_string()))?;

        match odbc_conn.execute(&sql, (), None) {
            Ok(Some(mut cursor)) => {
                if let Some(mut row) = cursor.next_row().map_err(|e| Error::Protocol(e.to_string()))? {
                    let mut buf = Vec::new();
                    let has_value = row
                        .get_text(1, &mut buf)
                        .map_err(|e| Error::Protocol(e.to_string()))?;

                    if has_value {
                        let s = String::from_utf8_lossy(&buf);
                        match s.trim().parse::<i64>() {
                            Ok(v) => Ok(Some(v)),
                            Err(_) => Ok(None),
                        }
                    } else {
                        Ok(None)
                    }
                } else {
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(Error::Protocol(e.to_string())),
        }
    })
    .await
    .map_err(|_| MigrateError::Source(Error::WorkerCrashed.into()))?
    .map_err(|e| MigrateError::Source(e.into()))?;

    Ok(result)
}

/// Execute a query and return migration rows (version, checksum)
async fn execute_query_migrations(
    conn: &mut OdbcConnection,
    sql: &str,
) -> Result<Vec<(i64, Vec<u8>)>, MigrateError> {
    let inner = conn.inner.clone();
    let sql = sql.to_string();

    let result = tokio::task::spawn_blocking(move || {
        let inner = inner.lock().map_err(|_| {
            Error::Protocol("Failed to lock ODBC connection".into())
        })?;

        let odbc_conn = inner
            .env
            .connect_with_connection_string(&inner.connection_string, odbc_api::ConnectionOptions::default())
            .map_err(|e| Error::Protocol(e.to_string()))?;

        match odbc_conn.execute(&sql, (), None) {
            Ok(Some(mut cursor)) => {
                let mut rows = Vec::new();

                while let Some(mut row) = cursor.next_row().map_err(|e| Error::Protocol(e.to_string()))? {
                    let mut version_buf = Vec::new();
                    let has_version = row
                        .get_text(1, &mut version_buf)
                        .map_err(|e| Error::Protocol(e.to_string()))?;

                    let version: i64 = if has_version {
                        String::from_utf8_lossy(&version_buf)
                            .trim()
                            .parse()
                            .unwrap_or(0)
                    } else {
                        continue;
                    };

                    let mut checksum_buf = Vec::new();
                    let has_checksum = row
                        .get_binary(2, &mut checksum_buf)
                        .map_err(|e| Error::Protocol(e.to_string()))?;

                    let checksum = if has_checksum { checksum_buf } else { Vec::new() };

                    rows.push((version, checksum));
                }

                Ok(rows)
            }
            Ok(None) => Ok(Vec::new()),
            Err(e) => Err(Error::Protocol(e.to_string())),
        }
    })
    .await
    .map_err(|_| MigrateError::Source(Error::WorkerCrashed.into()))?
    .map_err(|e| MigrateError::Source(e.into()))?;

    Ok(result)
}

/// Synchronous helper to execute a scalar string query
fn execute_scalar_string_sync(
    inner: Arc<Mutex<OdbcConnectionInner>>,
    sql: String,
) -> Result<Option<String>, Error> {
    let inner = inner.lock().map_err(|_| {
        Error::Protocol("Failed to lock ODBC connection".into())
    })?;

    let odbc_conn = inner
        .env
        .connect_with_connection_string(&inner.connection_string, odbc_api::ConnectionOptions::default())
        .map_err(|e| Error::Protocol(e.to_string()))?;

    match odbc_conn.execute(&sql, (), None) {
        Ok(Some(mut cursor)) => {
            if let Some(mut row) = cursor.next_row().map_err(|e| Error::Protocol(e.to_string()))? {
                let mut buf = Vec::new();
                let has_value = row
                    .get_text(1, &mut buf)
                    .map_err(|e| Error::Protocol(e.to_string()))?;

                if has_value {
                    Ok(Some(String::from_utf8_lossy(&buf).to_string()))
                } else {
                    Ok(None)
                }
            } else {
                Ok(None)
            }
        }
        Ok(None) => Ok(None),
        Err(e) => Err(Error::Protocol(e.to_string())),
    }
}
