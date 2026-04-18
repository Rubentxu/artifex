//! Repository construction helpers.

use std::sync::Arc;
use sqlx::SqlitePool;

use crate::repositories::{SqliteAssetRepository, SqliteJobRepository, SqliteProjectRepository};

/// Creates repository instances with the given database pool.
pub fn create_repositories(pool: SqlitePool) -> (
    Arc<SqliteProjectRepository>,
    Arc<SqliteJobRepository>,
    Arc<SqliteAssetRepository>,
) {
    let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));
    let job_repo = Arc::new(SqliteJobRepository::new(pool.clone()));
    let asset_repo = Arc::new(SqliteAssetRepository::new(pool.clone()));

    (project_repo, job_repo, asset_repo)
}
