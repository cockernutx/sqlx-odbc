//! Connection tests for ODBC driver with MS SQL Server.
//!
//! These tests require a running MS SQL Server instance.
//! Use `docker compose -f compose.dev.yaml up -d mssql` to start one.
//!
//! ## Known Limitations
//!
//! The current architecture creates a new ODBC connection for each operation.
//! This means:
//! - Temp tables don't persist between operations (they're connection-scoped)
//! - Transactions don't work across operations (BEGIN on one connection, COMMIT on another)
//!
//! Tests marked with `#[ignore]` document these limitations and will pass once
//! connection persistence is implemented.

use sqlx_odbc::odbc::{OdbcConnectOptions, OdbcConnection};
use sqlx_core::connection::Connection;
use sqlx_core::executor::Executor;

/// Get the connection string from environment or use default for local dev
fn get_connection_string() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "Driver={ODBC Driver 18 for SQL Server};Server=mssql,1433;Database=master;Uid=sa;Pwd=YourStrong@Passw0rd;TrustServerCertificate=yes".to_string()
    })
}

/// Helper to establish a connection for tests
async fn connect() -> OdbcConnection {
    let options = OdbcConnectOptions::new(get_connection_string());
    OdbcConnection::establish(&options)
        .await
        .expect("Failed to connect to database")
}

#[tokio::test]
async fn test_connection_establish() {
    let options = OdbcConnectOptions::new(get_connection_string());
    let conn = OdbcConnection::establish(&options).await;
    
    assert!(conn.is_ok(), "Should establish connection successfully");
    
    let conn = conn.unwrap();
    drop(conn);
}

#[tokio::test]
async fn test_connection_establish_invalid() {
    let options = OdbcConnectOptions::new(
        "Driver={ODBC Driver 18 for SQL Server};Server=nonexistent,1433;Database=master;Uid=sa;Pwd=wrong;TrustServerCertificate=yes"
    );
    
    let conn = OdbcConnection::establish(&options).await;
    assert!(conn.is_err(), "Should fail with invalid connection");
}

#[tokio::test]
async fn test_connection_ping() {
    let mut conn = connect().await;
    
    let result = conn.ping().await;
    assert!(result.is_ok(), "Ping should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_connection_close() {
    let conn = connect().await;
    
    let result = conn.close().await;
    assert!(result.is_ok(), "Close should succeed");
}

#[tokio::test]
async fn test_connection_close_hard() {
    let conn = connect().await;
    
    let result = conn.close_hard().await;
    assert!(result.is_ok(), "Close hard should succeed");
}

#[tokio::test]
async fn test_execute_simple_query() {
    let mut conn = connect().await;
    
    let result = conn.execute_raw("SELECT 1 AS value").await;
    assert!(result.is_ok(), "Simple query should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_execute_select_version() {
    let mut conn = connect().await;
    
    let result = conn.execute_raw("SELECT @@VERSION").await;
    assert!(result.is_ok(), "Version query should succeed: {:?}", result.err());
}

// Note: Transaction tests are ignored because the current architecture
// creates a new ODBC connection for each operation, so BEGIN TRANSACTION
// happens on a different connection than COMMIT/ROLLBACK.
#[tokio::test]
#[ignore = "Requires connection persistence - currently each operation uses a new connection"]
async fn test_transaction_begin_commit() {
    let mut conn = connect().await;
    
    // Begin transaction
    let tx = conn.begin().await;
    assert!(tx.is_ok(), "Begin transaction should succeed: {:?}", tx.err());
    
    let tx = tx.unwrap();
    
    // Commit
    let result = tx.commit().await;
    assert!(result.is_ok(), "Commit should succeed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "Requires connection persistence - currently each operation uses a new connection"]
async fn test_transaction_begin_rollback() {
    let mut conn = connect().await;
    
    // Begin transaction
    let tx = conn.begin().await;
    assert!(tx.is_ok(), "Begin transaction should succeed: {:?}", tx.err());
    
    let tx = tx.unwrap();
    
    // Rollback
    let result = tx.rollback().await;
    assert!(result.is_ok(), "Rollback should succeed: {:?}", result.err());
}

#[tokio::test]
#[ignore = "Requires connection persistence - currently each operation uses a new connection"]
async fn test_transaction_with_operations() {
    let mut conn = connect().await;
    
    // Create a temp table
    conn.execute_raw("CREATE TABLE #test_tx (id INT, name NVARCHAR(50))")
        .await
        .expect("Create temp table should succeed");
    
    // Begin transaction
    let mut tx = conn.begin().await.expect("Begin should succeed");
    
    // Insert data within transaction
    tx.execute_raw("INSERT INTO #test_tx VALUES (1, 'test')")
        .await
        .expect("Insert should succeed");
    
    // Rollback
    tx.rollback().await.expect("Rollback should succeed");
    
    // Note: Temp table is scoped to connection, but data was rolled back
}

#[tokio::test]
async fn test_multiple_queries_same_connection() {
    let mut conn = connect().await;
    
    for i in 0..5 {
        let result = conn.execute_raw(&format!("SELECT {} AS iteration", i)).await;
        assert!(result.is_ok(), "Query {} should succeed: {:?}", i, result.err());
    }
}

#[tokio::test]
async fn test_connection_options_builder() {
    let options = OdbcConnectOptions::new("DSN=test")
        .connection_string(get_connection_string());
    
    assert_eq!(options.get_connection_string(), get_connection_string());
}

#[tokio::test]
async fn test_fetch_single_row() {
    let mut conn = connect().await;
    
    let row = conn.fetch_optional("SELECT 1 AS num, 'hello' AS greeting").await;
    
    assert!(row.is_ok(), "Fetch should succeed: {:?}", row.err());
    let row = row.unwrap();
    assert!(row.is_some(), "Should return one row");
}

#[tokio::test]
async fn test_fetch_no_rows() {
    let mut conn = connect().await;
    
    let row = conn.fetch_optional("SELECT 1 WHERE 1 = 0").await;
    
    assert!(row.is_ok(), "Fetch should succeed: {:?}", row.err());
    let row = row.unwrap();
    assert!(row.is_none(), "Should return no rows");
}

// Note: Temp tables (#) and global temp tables (##) are session/connection-scoped,
// so they don't persist between operations in the current architecture.
// Using real tables with unique names and cleanup for testing.
#[tokio::test]
async fn test_create_and_query_table() {
    let mut conn = connect().await;
    
    // Use a unique table name to avoid conflicts between test runs
    let table_name = format!("test_data_{}", std::process::id());
    
    // Drop if exists from previous run
    let _ = conn.execute_raw(&format!("DROP TABLE IF EXISTS {}", table_name)).await;
    
    // Create table
    conn.execute_raw(&format!(
        "CREATE TABLE {} (id INT PRIMARY KEY, value NVARCHAR(100))", 
        table_name
    ))
        .await
        .expect("Create table should succeed");
    
    // Insert data
    conn.execute_raw(&format!(
        "INSERT INTO {} VALUES (1, 'first'), (2, 'second'), (3, 'third')",
        table_name
    ))
        .await
        .expect("Insert should succeed");
    
    // Query data
    let query = format!("SELECT * FROM {} WHERE id = 2", table_name);
    let row = conn.fetch_optional(query.as_str()).await;
    assert!(row.is_ok(), "Query should succeed: {:?}", row.err());
    assert!(row.unwrap().is_some(), "Should find row with id=2");
    
    // Cleanup
    conn.execute_raw(&format!("DROP TABLE IF EXISTS {}", table_name))
        .await
        .expect("Cleanup should succeed");
}

#[tokio::test]
async fn test_error_on_invalid_sql() {
    let mut conn = connect().await;
    
    let result = conn.execute_raw("THIS IS NOT VALID SQL").await;
    assert!(result.is_err(), "Invalid SQL should return error");
}

#[tokio::test]
async fn test_error_on_missing_table() {
    let mut conn = connect().await;
    
    let result = conn.execute_raw("SELECT * FROM nonexistent_table_xyz").await;
    assert!(result.is_err(), "Query on missing table should return error");
}

#[tokio::test]
async fn test_null_handling() {
    let mut conn = connect().await;
    
    let row = conn.fetch_optional("SELECT NULL AS null_value, 1 AS not_null").await;
    
    assert!(row.is_ok(), "Fetch should succeed: {:?}", row.err());
    assert!(row.unwrap().is_some(), "Should return a row");
}

#[tokio::test]
async fn test_unicode_data() {
    let mut conn = connect().await;
    
    // Use real table with unique name
    let table_name = format!("unicode_test_{}", std::process::id());
    
    // Drop if exists from previous run
    let _ = conn.execute_raw(&format!("DROP TABLE IF EXISTS {}", table_name)).await;
    
    // Create table with unicode data
    conn.execute_raw(&format!("CREATE TABLE {} (text NVARCHAR(100))", table_name))
        .await
        .expect("Create table should succeed");
    
    conn.execute_raw(&format!("INSERT INTO {} VALUES (N'Hello 世界')", table_name))
        .await
        .expect("Insert unicode should succeed");
    
    let query = format!("SELECT * FROM {}", table_name);
    let row = conn.fetch_optional(query.as_str()).await;
    assert!(row.is_ok(), "Query should succeed: {:?}", row.err());
    assert!(row.unwrap().is_some(), "Should return unicode row");
    
    // Cleanup
    conn.execute_raw(&format!("DROP TABLE IF EXISTS {}", table_name))
        .await
        .expect("Cleanup should succeed");
}

#[tokio::test]
async fn test_large_result_set() {
    let mut conn = connect().await;
    
    // Generate numbers 1-100 using SQL Server's recursive CTE
    let result = conn.fetch_optional(
        "WITH nums AS (
            SELECT 1 AS n
            UNION ALL
            SELECT n + 1 FROM nums WHERE n < 100
        )
        SELECT COUNT(*) AS cnt FROM nums OPTION (MAXRECURSION 100)"
    ).await;
    
    assert!(result.is_ok(), "Large query should succeed: {:?}", result.err());
}

#[tokio::test]
async fn test_multiple_columns() {
    let mut conn = connect().await;
    
    let row = conn.fetch_optional(
        "SELECT 
            1 AS int_col,
            'text' AS varchar_col,
            CAST(3.14 AS FLOAT) AS float_col,
            GETDATE() AS date_col,
            CAST(1 AS BIT) AS bit_col"
    ).await;
    
    assert!(row.is_ok(), "Multi-column query should succeed: {:?}", row.err());
    assert!(row.unwrap().is_some(), "Should return a row");
}

#[tokio::test]
async fn test_concurrent_connections() {
    let handles: Vec<_> = (0..3)
        .map(|i| {
            tokio::spawn(async move {
                let mut conn = connect().await;
                conn.execute_raw(&format!("SELECT {} AS conn_id", i))
                    .await
                    .expect("Query should succeed");
            })
        })
        .collect();
    
    for handle in handles {
        handle.await.expect("Task should complete");
    }
}

#[tokio::test]
async fn test_connection_debug_format() {
    let conn = connect().await;
    
    let debug_str = format!("{:?}", conn);
    assert!(debug_str.contains("OdbcConnection"), "Debug should show struct name");
    assert!(debug_str.contains("transaction_depth"), "Debug should show transaction_depth");
}
