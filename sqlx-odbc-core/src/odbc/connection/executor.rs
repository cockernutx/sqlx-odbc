//! Executor implementation for ODBC connections.

use crate::odbc::{
    Odbc, OdbcColumn, OdbcConnection, OdbcQueryResult, OdbcRow, OdbcStatement,
    OdbcTypeInfo, OdbcValue, OdbcValueData,
};
use futures_core::stream::BoxStream;
use futures_util::StreamExt;
use odbc_api::{Cursor, ResultSetMetadata, SharedConnection};
use sqlx_core::describe::Describe;
use sqlx_core::error::Error;
use sqlx_core::executor::{Execute, Executor};
use sqlx_core::Either;
use std::future::Future;
use std::pin::Pin;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

impl<'c> Executor<'c> for &'c mut OdbcConnection {
    type Database = Odbc;

    fn fetch_many<'e, 'q: 'e, E>(
        self,
        mut query: E,
    ) -> BoxStream<'e, Result<Either<OdbcQueryResult, OdbcRow>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        let sql = query.sql().to_string();
        let _arguments = query.take_arguments();
        let conn = self.conn.clone();

        Box::pin(async_stream::try_stream! {
            let rows = execute_query(conn, sql).await?;
            for row in rows {
                yield Either::Right(row);
            }
        })
    }

    fn fetch_optional<'e, 'q: 'e, E>(
        self,
        query: E,
    ) -> BoxFuture<'e, Result<Option<OdbcRow>, Error>>
    where
        'c: 'e,
        E: 'q + Execute<'q, Self::Database>,
    {
        Box::pin(async move {
            let mut stream = self.fetch_many(query);
            while let Some(result) = stream.next().await {
                match result? {
                    Either::Right(row) => return Ok(Some(row)),
                    Either::Left(_) => continue,
                }
            }
            Ok(None)
        })
    }

    fn prepare_with<'e, 'q: 'e>(
        self,
        sql: &'q str,
        _parameters: &'e [OdbcTypeInfo],
    ) -> BoxFuture<'e, Result<OdbcStatement<'q>, Error>>
    where
        'c: 'e,
    {
        Box::pin(async move {
            // For ODBC, we create a statement without actually preparing it on the server
            // The actual preparation happens on execute
            Ok(OdbcStatement::new(sql))
        })
    }

    fn describe<'e, 'q: 'e>(
        self,
        sql: &'q str,
    ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>>
    where
        'c: 'e,
    {
        let sql = sql.to_string();
        let conn = self.conn.clone();

        Box::pin(async move {
            let result = tokio::task::spawn_blocking(move || {
                describe_query(conn, sql)
            })
            .await
            .map_err(|_| Error::WorkerCrashed)??;

            Ok(result)
        })
    }
}

/// Execute a query and return rows (using persistent connection)
async fn execute_query(
    conn: SharedConnection<'static>,
    sql: String,
) -> Result<Vec<OdbcRow>, Error> {
    tokio::task::spawn_blocking(move || {
        let conn_guard = conn.lock().map_err(|_| {
            Error::Protocol("Failed to lock ODBC connection".into())
        })?;

        // Execute the query using the persistent connection
        match conn_guard.execute(&sql, (), None) {
            Ok(Some(mut cursor)) => {
                let mut rows = Vec::new();
                
                // Get column info
                let num_cols = cursor.num_result_cols()
                    .map_err(|e| Error::Protocol(e.to_string()))? as usize;
                
                let mut columns = Vec::with_capacity(num_cols);
                for i in 1..=num_cols {
                    let mut desc = odbc_api::ColumnDescription::default();
                    cursor.describe_col(i as u16, &mut desc)
                        .map_err(|e| Error::Protocol(e.to_string()))?;
                    
                    let name = String::from_utf8_lossy(&desc.name).to_string();
                    columns.push(OdbcColumn::new(
                        i - 1,
                        name,
                        OdbcTypeInfo::new(desc.data_type),
                    ));
                }

                // Fetch all rows
                while let Some(mut row) = cursor.next_row()
                    .map_err(|e| Error::Protocol(e.to_string()))? 
                {
                    let mut values = Vec::with_capacity(num_cols);
                    for i in 1..=num_cols {
                        // Read as string for simplicity (should use proper type handling)
                        let mut buf = Vec::new();
                        let has_value = row.get_text(i as u16, &mut buf)
                            .map_err(|e| Error::Protocol(e.to_string()))?;
                        
                        let data = if has_value {
                            OdbcValueData::Text(String::from_utf8_lossy(&buf).to_string())
                        } else {
                            OdbcValueData::Null
                        };
                        
                        values.push(OdbcValue::new(
                            data,
                            columns[i - 1].type_info.clone(),
                        ));
                    }
                    rows.push(OdbcRow::new(columns.clone(), values));
                }

                Ok(rows)
            }
            Ok(None) => Ok(Vec::new()),
            Err(e) => Err(Error::Protocol(e.to_string())),
        }
    })
    .await
    .map_err(|_| Error::WorkerCrashed)?
}

/// Describe a query to get column and parameter info (using persistent connection)
fn describe_query(
    conn: SharedConnection<'static>,
    sql: String,
) -> Result<Describe<Odbc>, Error> {
    let conn_guard = conn.lock().map_err(|_| {
        Error::Protocol("Failed to lock ODBC connection".into())
    })?;

    // Prepare the statement to get metadata using persistent connection
    let mut prepared = conn_guard.prepare(&sql)
        .map_err(|e| Error::Protocol(e.to_string()))?;

    // Get column information
    let num_cols = prepared.num_result_cols()
        .map_err(|e| Error::Protocol(e.to_string()))? as usize;

    let mut columns = Vec::with_capacity(num_cols);
    let mut nullable = Vec::with_capacity(num_cols);

    for i in 1..=num_cols {
        let mut desc = odbc_api::ColumnDescription::default();
        prepared.describe_col(i as u16, &mut desc)
            .map_err(|e| Error::Protocol(e.to_string()))?;

        let name = String::from_utf8_lossy(&desc.name).to_string();
        columns.push(OdbcColumn::new(
            i - 1,
            name,
            OdbcTypeInfo::new(desc.data_type),
        ));
        nullable.push(Some(desc.nullability.could_be_nullable()));
    }

    // Get parameter information
    let num_params = prepared.num_params()
        .map_err(|e| Error::Protocol(e.to_string()))? as usize;

    let mut parameters = Vec::with_capacity(num_params);
    for i in 1..=num_params {
        let param_desc = prepared.describe_param(i as u16)
            .map_err(|e| Error::Protocol(e.to_string()))?;
        parameters.push(OdbcTypeInfo::new(param_desc.data_type));
    }

    Ok(Describe {
        columns,
        parameters: Some(Either::Left(parameters)),
        nullable,
    })
}
