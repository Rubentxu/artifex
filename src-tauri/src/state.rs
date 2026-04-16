//! Application state managed by Tauri.

use std::sync::{Arc, Mutex};

use crate::application::asset_service::AssetApplicationService;
use crate::application::job_service::JobApplicationService;
use crate::application::project_service::ProjectApplicationService;
use crate::model_config::ModelConfigService;
use crate::workers::WorkerRunner;

/// Application state holding application services.
pub struct AppState {
    /// Project application service.
    pub service: Arc<ProjectApplicationService>,
    /// Job application service.
    pub job_service: Arc<JobApplicationService>,
    /// Asset application service.
    pub asset_service: Arc<AssetApplicationService>,
    /// Currently open project ID, if any.
    pub current_project_id: Mutex<Option<String>>,
    /// Worker runner for processing jobs.
    pub worker_runner: Arc<Mutex<Option<WorkerRunner>>>,
    /// Model configuration service.
    pub model_config_service: Arc<ModelConfigService>,
}

impl AppState {
    /// Creates a new AppState.
    pub fn new(
        service: Arc<ProjectApplicationService>,
        job_service: Arc<JobApplicationService>,
        asset_service: Arc<AssetApplicationService>,
        current_project_id: Mutex<Option<String>>,
        worker_runner: Arc<Mutex<Option<WorkerRunner>>>,
        model_config_service: Arc<ModelConfigService>,
    ) -> Self {
        Self {
            service,
            job_service,
            asset_service,
            current_project_id,
            worker_runner,
            model_config_service,
        }
    }
}
