//! Application state construction.

use std::sync::{Arc, Mutex};

use crate::application::{AssetApplicationService, JobApplicationService, ProjectApplicationService};
use crate::model_config::ModelConfigService;
use crate::state::AppState;
use crate::workers::WorkerRunner;

/// Creates the application state.
pub fn create_app_state(
    project_service: Arc<ProjectApplicationService>,
    job_service: Arc<JobApplicationService>,
    asset_service: Arc<AssetApplicationService>,
    worker_runner: WorkerRunner,
    model_config_service: Arc<ModelConfigService>,
) -> AppState {
    let runner_handle = Arc::new(Mutex::new(Some(worker_runner)));

    AppState::new(
        project_service,
        job_service,
        asset_service,
        Mutex::new(None),
        runner_handle,
        model_config_service,
    )
}
