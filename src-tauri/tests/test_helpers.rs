//! Test helpers for src-tauri integration tests.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

/// Creates a test database pool with migrations applied.
pub async fn setup_test_db() -> Result<SqlitePool, sqlx::Error> {
    // Use a temporary in-memory database for tests
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}


