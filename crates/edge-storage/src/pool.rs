//! Connection pooling and migration runner configuration.

use crate::error::StorageError;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

/// PostgreSQL connection pool configuration.
#[derive(Debug, Clone)]
pub struct PgPoolConfig {
    /// Connection string URI (e.g. `postgres://user:pass@host:5432/db`).
    pub database_url: String,
    /// Maximum number of connections in the pool.
    pub max_connections: u32,
    /// Minimum number of idle connections maintained.
    pub min_connections: u32,
    /// Connection acquisition timeout.
    pub connect_timeout: Duration,
    /// Max idle lifetime before closing an idle connection.
    pub idle_timeout: Duration,
}

impl Default for PgPoolConfig {
    fn default() -> Self {
        Self {
            database_url: "postgres://postgres:postgres@localhost:5432/edge_arena".to_string(),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(600),
        }
    }
}

impl PgPoolConfig {
    /// Constructs a configuration with the given database URL and default settings.
    #[must_use]
    pub fn new(database_url: impl Into<String>) -> Self {
        Self {
            database_url: database_url.into(),
            ..Default::default()
        }
    }

    /// Creates an asynchronous `PgPool` based on the specified parameters.
    pub async fn create_pool(&self) -> Result<PgPool, StorageError> {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .acquire_timeout(self.connect_timeout)
            .idle_timeout(self.idle_timeout)
            .connect(&self.database_url)
            .await
            .map_err(|e| StorageError::PoolConfiguration(e.to_string()))
    }
}

/// Executes all pending SQL migrations against the target PostgreSQL pool.
pub async fn run_migrations(pool: &PgPool) -> Result<(), StorageError> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| StorageError::Migration(e.to_string()))
}
