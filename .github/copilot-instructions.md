# SQLx-ODBC Development Guide

## Project Overview

This is an ODBC database driver for SQLx, providing generic connectivity to any ODBC-compatible database through the `odbc-api` crate. The project implements the SQLx `Database` trait to enable async database operations over synchronous ODBC APIs.

**Key Architecture Pattern**: Async wrapper over sync ODBC - all ODBC operations are wrapped in `tokio::task::spawn_blocking` to avoid blocking the async runtime. The connection is persistent and shared via `SharedConnection<'static>` (which wraps `Arc<Mutex<Connection>>`).

## Project Structure

- `sqlx-odbc-core/src/odbc/` - Main ODBC driver implementation (current/active)
- `sqlx-odbc-core/src/odbc_old/` - Previous implementation (legacy, for reference)
- `Containerfile.dev` - Dev container with MS SQL Server ODBC driver installed
- `compose.dev.yaml` - Docker Compose setup with SQL Server test database

## Critical Architectural Patterns

### 1. Async-over-Sync Bridge Pattern

ODBC APIs are synchronous, but SQLx requires async. **Every ODBC operation MUST use `tokio::task::spawn_blocking`**:

```rust
// Example from connection/mod.rs - the with_conn helper method
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
```

**Why**: ODBC calls can block for extended periods (network I/O, query execution). Running them on the async runtime would stall other tasks.

### 2. Persistent Connection with SharedConnection

The ODBC connection is persistent and wrapped in `SharedConnection<'static>` for thread-safe access:

```rust
/// A connection to an ODBC-accessible database.
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
```

`SharedConnection<'static>` is `odbc_api::SharedConnection` which internally uses `Arc<Mutex<Connection>>`. This allows:
- **Connection persistence** - The same ODBC connection is reused across all operations
- **Temp table support** - Session-scoped temp tables (`#table_name`) persist across queries
- **Transaction support** - `BEGIN`/`COMMIT`/`ROLLBACK` work correctly on the same connection
- **Thread-safe access** - The mutex ensures safe concurrent access from async tasks

### 3. Type System Implementation

All ODBC type conversions follow this triadic pattern in `types/mod.rs`:

```rust
impl Type<Odbc> for T { ... }           // Declare type mapping
impl Encode<'q, Odbc> for T { ... }     // Rust → ODBC
impl Decode<'r, Odbc> for T { ... }     // ODBC → Rust
```

Values flow through `OdbcArgumentValue` (encoding) and `OdbcValueData` (decoding). Both are enums covering all supported SQL types.

## Development Workflows

### Building

```bash
cargo build
```

The workspace uses Rust edition 2024. The main crate is `sqlx-odbc-core`.

### Testing with SQL Server

The dev container includes ODBC Driver 18 for SQL Server:

```bash
# Start SQL Server
docker compose -f compose.dev.yaml up -d mssql

# Connection string format (from compose.dev.yaml)
Driver={ODBC Driver 18 for SQL Server};Server=mssql,1433;Database=master;Uid=sa;Pwd=YourStrong@Passw0rd;TrustServerCertificate=yes
```

### Dev Container Environment

- Pre-installed: ODBC Driver 18 for SQL Server, mssql-tools18, unixodbc-dev
- VS Code extensions: rust-analyzer, vscode-lldb, even-better-toml
- SQL Server available at `mssql:1433` from within container

## Project-Specific Conventions

### Error Handling

Convert ODBC errors through `OdbcDatabaseError::new()` which extracts SQLSTATE codes:

```rust
// From error.rs
impl DatabaseError for OdbcDatabaseError {
    fn kind(&self) -> ErrorKind {
        match self.sqlstate.as_deref() {
            Some(s) if s.starts_with("23") => { /* Constraint violations */ }
            // ...
        }
    }
}
```

### Connection Options

Buffer settings control fetch behavior (`options.rs`):

- `batch_size`: Rows fetched at once (default: 128)
- `max_column_size`: Text/binary column limit. `None` = unbuffered mode (no truncation)

### Transaction Management

Transactions use ODBC's native autocommit control (`transaction.rs`):
- `conn.set_autocommit(false)` - begin transaction
- `conn.commit()` + `conn.set_autocommit(true)` - commit
- `conn.rollback()` + `conn.set_autocommit(true)` - rollback

Track depth with `conn.transaction_depth` for nested transaction support.

### Statement Preparation

ODBC statements are lazily prepared - `prepare_with()` returns immediately, actual preparation happens on first execute (see `connection/executor.rs`).

## Integration Points

### SQLx Core Traits

Implement these key traits to integrate with SQLx:

- `Database` - Core database definition (`database.rs`)
- `Connection` - Connection lifecycle (`connection/mod.rs`)
- `Executor` - Query execution (`connection/executor.rs`)
- `TransactionManager` - Transaction handling (`transaction.rs`)
- `Arguments` - Parameter binding (`arguments.rs`)
- `Row`, `Column`, `TypeInfo`, `Value` - Result handling

### External Dependencies

- `odbc-api` (v20.1.0) - ODBC bindings
- `sqlx-core` (v0.8.6) - SQLx database trait definitions
- `tokio` - Async runtime with `spawn_blocking` for sync operations
- `async-stream` - Stream implementation for row iteration

## Common Pitfalls

1. **Don't call ODBC APIs directly in async functions** - always wrap in `spawn_blocking`
2. **Use `with_conn` helper** - it handles locking and `spawn_blocking` for you
3. **Connection strings must include driver name** - e.g., `Driver={ODBC Driver 18 for SQL Server}`
4. **SQLSTATE extraction requires string parsing** - use `extract_sqlstate()` in `error.rs`
5. **Placeholder format is `?`** - not `$1`, `$2` like PostgreSQL (see `arguments.rs`)

## Key Files for Reference

- `connection/mod.rs` - Connection establishment, `with_conn` helper, transaction methods
- `connection/executor.rs` - Query execution, row fetching, `spawn_blocking` usage
- `database.rs` - `OdbcArgumentValue` enum (all supported types)
- `types/mod.rs` - Type encoding/decoding examples
- `error.rs` - SQLSTATE to ErrorKind mapping

## Database-Specific Migration Support

Migration support is implemented as **extension traits** that users must explicitly import. Each database has its own feature flag and module.

### Feature Flags

```toml
# Cargo.toml
[features]
mssql-migrate = ["sqlx-core/migrate", "crc"]
postgres-migrate = ["sqlx-core/migrate", "crc"]
```

### MSSQL Migrations

Enable with `--features mssql-migrate`. Located in `sqlx-odbc-core/src/odbc/mssql/`.

```rust
use sqlx_odbc::odbc::mssql::{MssqlMigrateExt, MssqlMigrateDatabaseExt};

// Database operations (via MssqlMigrateDatabaseExt)
conn.mssql_database_exists("mydb").await?;
conn.mssql_create_database("mydb").await?;
conn.mssql_drop_database("mydb").await?;

// Migration operations (via MssqlMigrateExt)
conn.mssql_ensure_migrations_table().await?;
conn.mssql_lock().await?;
conn.mssql_run_migrations(&migrator).await?;
conn.mssql_unlock().await?;
```

**MSSQL-specific details:**
- Uses `sp_getapplock` / `sp_releaseapplock` for advisory locking
- Queries `sys.databases` to check database existence
- Uses bracket escaping `[database_name]` for identifiers
- Transaction syntax: `BEGIN TRANSACTION` / `COMMIT` / `ROLLBACK`

### PostgreSQL Migrations

Enable with `--features postgres-migrate`. Located in `sqlx-odbc-core/src/odbc/postgres/`.

```rust
use sqlx_odbc::odbc::postgres::{PostgresMigrateExt, PostgresMigrateDatabaseExt};

// Database operations (via PostgresMigrateDatabaseExt)
conn.postgres_database_exists("mydb").await?;
conn.postgres_create_database("mydb").await?;
conn.postgres_drop_database("mydb").await?;  // Force drops with pg_terminate_backend

// Migration operations (via PostgresMigrateExt)
conn.postgres_ensure_migrations_table().await?;
conn.postgres_lock().await?;
conn.postgres_run_migrations(&migrator).await?;
conn.postgres_unlock().await?;
```

**PostgreSQL-specific details:**
- Uses `pg_advisory_lock` / `pg_advisory_unlock` for advisory locking
- Queries `pg_database` to check database existence
- Uses double-quote escaping `"database_name"` for identifiers
- Connects to `template1` or `postgres` for maintenance operations
- Uses `pg_terminate_backend` to force-drop databases with active connections
- Transaction syntax: `BEGIN` / `COMMIT` / `ROLLBACK`

### Extension Trait Pattern

Migration support uses extension traits rather than implementing SQLx's `Migrate` trait directly. This design:

1. **Requires explicit import** - Users must `use` the extension trait to access methods
2. **Database-specific naming** - Methods are prefixed (`mssql_`, `postgres_`) to avoid conflicts
3. **Optional via features** - Only compiled when the corresponding feature is enabled
4. **No trait conflicts** - Avoids issues with SQLx's `Migrate` trait expectations

### Adding New Database Migration Support

1. Create a new module: `sqlx-odbc-core/src/odbc/{database}/`
2. Add feature flag in `Cargo.toml`: `{database}-migrate = ["sqlx-core/migrate", "crc"]`
3. Implement extension traits following the pattern in `mssql/migrate.rs` or `postgres/migrate.rs`
4. Add conditional export in `sqlx-odbc-core/src/odbc/mod.rs`

Key considerations for new databases:
- Advisory locking mechanism (database-specific)
- Identifier escaping rules
- System catalog queries for database existence
- Transaction syntax differences
- Maintenance database for create/drop operations
