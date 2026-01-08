//! Database connection and management.

use crate::error::{StorageError, StorageResult};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::path::Path;
use std::str::FromStr;
use tracing::{debug, info};

/// Database configuration.
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    /// Database file path
    pub path: String,
    /// Maximum number of connections
    pub max_connections: u32,
    /// Create database if it doesn't exist
    pub create_if_missing: bool,
    /// Run migrations on connect
    pub run_migrations: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            path: crate::default_database_path()
                .to_string_lossy()
                .to_string(),
            max_connections: 5,
            create_if_missing: true,
            run_migrations: true,
        }
    }
}

impl DatabaseConfig {
    /// Create config for in-memory database (useful for testing).
    pub fn in_memory() -> Self {
        Self {
            path: ":memory:".to_string(),
            max_connections: 1,
            create_if_missing: true,
            run_migrations: true,
        }
    }

    /// Create config with custom path.
    pub fn with_path(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            ..Default::default()
        }
    }
}

/// Database connection pool and operations.
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Connect to the database with the given configuration.
    pub async fn connect(config: &DatabaseConfig) -> StorageResult<Self> {
        debug!("Connecting to database: {}", config.path);

        // Ensure parent directory exists
        if config.path != ":memory:" {
            if let Some(parent) = Path::new(&config.path).parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
        }

        let options = SqliteConnectOptions::from_str(&config.path)
            .map_err(|e| StorageError::Connection(e.to_string()))?
            .create_if_missing(config.create_if_missing)
            .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal)
            .synchronous(sqlx::sqlite::SqliteSynchronous::Normal)
            .busy_timeout(std::time::Duration::from_secs(30));

        let pool = SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .connect_with(options)
            .await?;

        let db = Self { pool };

        if config.run_migrations {
            db.run_migrations().await?;
        }

        info!("Database connected: {}", config.path);
        Ok(db)
    }

    /// Run database migrations.
    pub async fn run_migrations(&self) -> StorageResult<()> {
        debug!("Running database migrations");

        // Create tables
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS diagnostic_sessions (
                id TEXT PRIMARY KEY,
                started_at TEXT NOT NULL,
                ended_at TEXT,
                session_type TEXT NOT NULL,
                status TEXT NOT NULL,
                summary TEXT,
                metadata TEXT
            );

            CREATE TABLE IF NOT EXISTS ping_results (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                target TEXT NOT NULL,
                resolved_ip TEXT,
                transmitted INTEGER NOT NULL,
                received INTEGER NOT NULL,
                loss_percent REAL NOT NULL,
                min_rtt_ms REAL,
                avg_rtt_ms REAL,
                max_rtt_ms REAL,
                stddev_ms REAL,
                quality TEXT,
                created_at TEXT NOT NULL,
                raw_data TEXT,
                FOREIGN KEY (session_id) REFERENCES diagnostic_sessions(id)
            );

            CREATE TABLE IF NOT EXISTS traceroute_results (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                target TEXT NOT NULL,
                resolved_ip TEXT,
                hop_count INTEGER NOT NULL,
                reached INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                protocol TEXT NOT NULL,
                created_at TEXT NOT NULL,
                hops_data TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES diagnostic_sessions(id)
            );

            CREATE TABLE IF NOT EXISTS speed_results (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                server TEXT,
                download_mbps REAL,
                upload_mbps REAL,
                ping_ms REAL,
                jitter_ms REAL,
                created_at TEXT NOT NULL,
                raw_data TEXT,
                FOREIGN KEY (session_id) REFERENCES diagnostic_sessions(id)
            );

            CREATE TABLE IF NOT EXISTS interface_snapshots (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                name TEXT NOT NULL,
                interface_type TEXT NOT NULL,
                mac_address TEXT,
                ipv4_addresses TEXT NOT NULL,
                ipv6_addresses TEXT NOT NULL,
                is_up INTEGER NOT NULL,
                is_default INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES diagnostic_sessions(id)
            );

            CREATE TABLE IF NOT EXISTS dns_results (
                id TEXT PRIMARY KEY,
                session_id TEXT,
                query TEXT NOT NULL,
                addresses TEXT NOT NULL,
                duration_ms REAL NOT NULL,
                success INTEGER NOT NULL,
                error TEXT,
                created_at TEXT NOT NULL,
                FOREIGN KEY (session_id) REFERENCES diagnostic_sessions(id)
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes
        sqlx::query(
            r#"
            CREATE INDEX IF NOT EXISTS idx_ping_created_at ON ping_results(created_at);
            CREATE INDEX IF NOT EXISTS idx_ping_target ON ping_results(target);
            CREATE INDEX IF NOT EXISTS idx_ping_session ON ping_results(session_id);
            CREATE INDEX IF NOT EXISTS idx_traceroute_created_at ON traceroute_results(created_at);
            CREATE INDEX IF NOT EXISTS idx_traceroute_target ON traceroute_results(target);
            CREATE INDEX IF NOT EXISTS idx_speed_created_at ON speed_results(created_at);
            CREATE INDEX IF NOT EXISTS idx_dns_created_at ON dns_results(created_at);
            CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON diagnostic_sessions(started_at);
            "#,
        )
        .execute(&self.pool)
        .await?;

        info!("Database migrations completed");
        Ok(())
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Close the database connection.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Check if the database is connected.
    pub fn is_closed(&self) -> bool {
        self.pool.is_closed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_database() {
        let config = DatabaseConfig::in_memory();
        let db = Database::connect(&config).await.unwrap();
        assert!(!db.is_closed());
        db.close().await;
    }
}
