//! Application setup helpers.

use std::path::PathBuf;
use std::sync::Mutex;

use tauri::Manager;

use crate::application::{AssetApplicationService, JobApplicationService, ProjectApplicationService};
use crate::bootstrap::database::init_database;
use crate::bootstrap::model_config::seed_model_config;
use crate::bootstrap::repositories::create_repositories;
use crate::bootstrap::services::create_services;
use crate::bootstrap::model_config::create_model_config;
use crate::bootstrap::workers::create_workers;
use crate::bootstrap::state::create_app_state;
use crate::model_config::ModelConfigService;
use crate::state::AppState;
use crate::workers::WorkerRunner;

/// Initializes the application with all state and workers set up.
pub fn setup_app(app: &tauri::App) -> Result<AppState, String> {
    let app_dir = app.path().app_data_dir().map_err(|e| {
        format!("Failed to resolve app data directory: {}", e)
    })?;

    std::fs::create_dir_all(&app_dir).map_err(|e| {
        format!("Failed to create app data directory: {}", e)
    })?;

    // Initialize database pool
    let pool = init_database(&app_dir)
        .map_err(|e| format!("Failed to initialize database: {}", e))?;

    // Create repositories
    let (project_repo, job_repo, asset_repo) = create_repositories(pool.clone());

    // Create application services
    let (project_service, job_service, asset_service) = create_services(
        project_repo.clone(),
        job_repo.clone(),
        asset_repo.clone(),
    );

    // Create model config infrastructure
    let (model_config_service, model_router, credential_store) = create_model_config(pool.clone());

    // Seed default model profiles and routing rules
    seed_model_config(&model_config_service);

    // Create worker infrastructure
    let assets_dir = app_dir.join("artifex-assets").to_string_lossy().to_string();
    let worker_runner = create_workers(
        model_router,
        credential_store,
        assets_dir,
        job_repo.clone(),
        asset_service.clone(),
        app.handle().clone(),
    );

    // Create application state
    let app_state = create_app_state(
        project_service,
        job_service,
        asset_service,
        worker_runner,
        model_config_service,
    );

    Ok(app_state)
}

/// Spawns the worker runner in the background.
pub fn spawn_worker_runner(app_state: &AppState) {
    let runner_handle = app_state.worker_runner.clone();

    tokio::spawn(async move {
        let runner = {
            let mut guard = runner_handle.lock()
                .expect("worker_runner lock poisoned");
            guard.take()
        };
        if let Some(r) = runner {
            r.run().await;
        }
    });
}
