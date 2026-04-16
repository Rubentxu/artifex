//! Database initialization and connection pool management.

use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::path::Path;

/// Initializes a SQLite connection pool at the given path and runs migrations.
pub async fn init_db_pool<P: AsRef<Path>>(db_path: P) -> Result<SqlitePool, sqlx::Error> {
    let db_url = format!(
        "sqlite:{}?mode=rwc",
        db_path.as_ref().display()
    );
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    // Run migrations from the migrations directory
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
