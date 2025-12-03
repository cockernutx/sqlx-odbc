//! ODBC connection options.

use crate::odbc::OdbcConnection;
use sqlx_core::connection::ConnectOptions;
use sqlx_core::error::Error;
use sqlx_core::Url;
use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::time::Duration;

/// Buffer settings for ODBC data fetching.
#[derive(Debug, Clone)]
pub struct OdbcBufferSettings {
    /// Number of rows to fetch at once in batch mode
    pub batch_size: usize,
    /// Maximum size for text/binary columns (None = unbuffered mode)
    pub max_column_size: Option<usize>,
}

impl Default for OdbcBufferSettings {
    fn default() -> Self {
        Self {
            batch_size: 128,
            max_column_size: Some(4096),
        }
    }
}

/// Options for connecting to an ODBC data source.
#[derive(Debug, Clone)]
pub struct OdbcConnectOptions {
    /// The ODBC connection string
    pub(crate) connection_string: String,
    /// Buffer settings for data fetching
    pub(crate) buffer_settings: OdbcBufferSettings,
    /// Connection timeout
    pub(crate) connect_timeout: Option<Duration>,
    /// Statement logging level
    pub(crate) log_statements: log::LevelFilter,
    /// Slow statement threshold
    pub(crate) log_slow_statements: (log::LevelFilter, Duration),
}

impl Default for OdbcConnectOptions {
    fn default() -> Self {
        Self {
            connection_string: String::new(),
            buffer_settings: OdbcBufferSettings::default(),
            connect_timeout: Some(Duration::from_secs(30)),
            log_statements: log::LevelFilter::Debug,
            log_slow_statements: (log::LevelFilter::Warn, Duration::from_secs(1)),
        }
    }
}

impl OdbcConnectOptions {
    /// Create new options with the given connection string
    pub fn new(connection_string: impl Into<String>) -> Self {
        Self {
            connection_string: connection_string.into(),
            ..Default::default()
        }
    }

    /// Create options from a DSN
    pub fn from_dsn(dsn: impl Into<String>) -> Self {
        let dsn = dsn.into();
        Self::new(format!("DSN={}", dsn))
    }

    /// Set the connection string
    pub fn connection_string(mut self, connection_string: impl Into<String>) -> Self {
        self.connection_string = connection_string.into();
        self
    }

    /// Get the connection string
    pub fn get_connection_string(&self) -> &str {
        &self.connection_string
    }

    /// Set the buffer settings
    pub fn buffer_settings(mut self, settings: OdbcBufferSettings) -> Self {
        self.buffer_settings = settings;
        self
    }

    /// Set the batch size for fetching rows
    pub fn batch_size(mut self, size: usize) -> Self {
        self.buffer_settings.batch_size = size;
        self
    }

    /// Set the maximum column size (None for unbuffered mode)
    pub fn max_column_size(mut self, size: Option<usize>) -> Self {
        self.buffer_settings.max_column_size = size;
        self
    }

    /// Set the connection timeout
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = Some(timeout);
        self
    }

    /// Add a username to the connection string
    pub fn username(mut self, username: &str) -> Self {
        if !self.connection_string.is_empty() {
            self.connection_string.push(';');
        }
        self.connection_string.push_str("UID=");
        self.connection_string.push_str(username);
        self
    }

    /// Add a password to the connection string
    pub fn password(mut self, password: &str) -> Self {
        if !self.connection_string.is_empty() {
            self.connection_string.push(';');
        }
        self.connection_string.push_str("PWD=");
        self.connection_string.push_str(password);
        self
    }

    /// Add a driver to the connection string
    pub fn driver(mut self, driver: &str) -> Self {
        if !self.connection_string.is_empty() {
            self.connection_string.push(';');
        }
        self.connection_string.push_str("Driver={");
        self.connection_string.push_str(driver);
        self.connection_string.push('}');
        self
    }

    /// Add a server to the connection string
    pub fn server(mut self, server: &str) -> Self {
        if !self.connection_string.is_empty() {
            self.connection_string.push(';');
        }
        self.connection_string.push_str("Server=");
        self.connection_string.push_str(server);
        self
    }

    /// Add a database to the connection string
    pub fn database(mut self, database: &str) -> Self {
        if !self.connection_string.is_empty() {
            self.connection_string.push(';');
        }
        self.connection_string.push_str("Database=");
        self.connection_string.push_str(database);
        self
    }
}

impl FromStr for OdbcConnectOptions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Handle URL-style connection strings
        if s.starts_with("odbc:") || s.starts_with("odbc://") {
            let conn_str = s
                .strip_prefix("odbc://")
                .or_else(|| s.strip_prefix("odbc:"))
                .unwrap_or(s);
            Ok(Self::new(conn_str))
        } else {
            // Assume it's a raw ODBC connection string
            Ok(Self::new(s))
        }
    }
}

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

impl ConnectOptions for OdbcConnectOptions {
    type Connection = OdbcConnection;

    fn from_url(url: &Url) -> Result<Self, Error> {
        // Convert URL to connection string
        let mut conn_string = String::new();

        // Extract DSN or driver from host
        if let Some(host) = url.host_str() {
            if host.starts_with('{') {
                conn_string.push_str("Driver=");
                conn_string.push_str(host);
            } else {
                conn_string.push_str("DSN=");
                conn_string.push_str(host);
            }
        }

        // Add database from path
        let path = url.path();
        if !path.is_empty() && path != "/" {
            let db = path.trim_start_matches('/');
            if !db.is_empty() {
                conn_string.push_str(";Database=");
                conn_string.push_str(db);
            }
        }

        // Add username
        if !url.username().is_empty() {
            conn_string.push_str(";UID=");
            conn_string.push_str(url.username());
        }

        // Add password
        if let Some(password) = url.password() {
            conn_string.push_str(";PWD=");
            conn_string.push_str(password);
        }

        // Add any query parameters as connection string options
        for (key, value) in url.query_pairs() {
            conn_string.push(';');
            conn_string.push_str(&key);
            conn_string.push('=');
            conn_string.push_str(&value);
        }

        Ok(Self::new(conn_string))
    }

    fn to_url_lossy(&self) -> Url {
        // This is a lossy conversion - we can't fully represent ODBC connection strings as URLs
        Url::parse(&format!("odbc:{}", self.connection_string))
            .unwrap_or_else(|_| Url::parse("odbc:").unwrap())
    }

    fn connect(&self) -> BoxFuture<'_, Result<Self::Connection, Error>>
    where
        Self::Connection: Sized,
    {
        Box::pin(async move { OdbcConnection::establish(self).await })
    }

    fn log_statements(mut self, level: log::LevelFilter) -> Self {
        self.log_statements = level;
        self
    }

    fn log_slow_statements(mut self, level: log::LevelFilter, duration: Duration) -> Self {
        self.log_slow_statements = (level, duration);
        self
    }
}
