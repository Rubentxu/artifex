//! Application service construction helpers.

use std::sync::Arc;

use crate::application::{AssetApplicationService, JobApplicationService, ProjectApplicationService};
use crate::repositories::{SqliteAssetRepository, SqliteJobRepository, SqliteProjectRepository};

/// Creates application service instances with the given repositories.
pub fn create_services(
    project_repo: Arc<SqliteProjectRepository>,
    job_repo: Arc<SqliteJobRepository>,
    asset_repo: Arc<SqliteAssetRepository>,
) -> (
    Arc<ProjectApplicationService>,
    Arc<JobApplicationService>,
    Arc<AssetApplicationService>,
) {
    let project_service = Arc::new(ProjectApplicationService::new(project_repo.clone()));
    let job_service = Arc::new(JobApplicationService::new(job_repo.clone()));
    let asset_service = Arc::new(AssetApplicationService::new(asset_repo.clone()));

    (project_service, job_service, asset_service)
}
