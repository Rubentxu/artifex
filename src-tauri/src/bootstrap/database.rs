//! Database initialization and pool management.

use std::path::PathBuf;
use sqlx::SqlitePool;

use crate::db;

/// Initializes the database pool for the application.
///
/// This function is called during Tauri setup to create the database
/// connection pool and run migrations.
pub fn init_database(app_dir: &PathBuf) -> Result<SqlitePool, String> {
    let db_path = app_dir.join("artifex.db");

    // Use block_in_place to allow blocking inside an async context
    // (tauri-driver launches the app within a tokio runtime).
    let pool = tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(db::init_db_pool(&db_path))
    }).map_err(|e| format!("Database initialization failed: {}", e))?;

    Ok(pool)
}
