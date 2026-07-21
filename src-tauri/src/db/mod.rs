pub mod schema;
pub mod migrations;

use sqlx::SqlitePool;
use migrations::run_migrations;

/// Represents the open Praetorian catalog (SQLite connection pool).
pub struct Database {
    pub pool: SqlitePool,
}

impl Database {
    /// Opens an existing `.praetorian` catalog or creates a new one.
    pub async fn open_or_create(path: &str) -> Result<Self, sqlx::Error> {
        let dsn = format!("sqlite:{}", path);
        let pool = SqlitePool::connect(&dsn).await?;

        // Enable WAL mode for better concurrent read/write performance
        sqlx::query("PRAGMA journal_mode = WAL;")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA foreign_keys = ON;")
            .execute(&pool)
            .await?;
        sqlx::query("PRAGMA cache_size = -64000;") // 64MB cache
            .execute(&pool)
            .await?;

        run_migrations(&pool).await?;

        Ok(Self { pool })
    }

    /// Returns the connection pool for use by other modules.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}
