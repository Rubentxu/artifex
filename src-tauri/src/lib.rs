//! Artifex library crate.
//!
//! This lib.rs contains all the modules including command definitions.

pub mod application;
pub mod commands;
pub mod db;
pub mod dto;
pub mod model_config;
pub mod repositories;
pub mod state;
pub mod workers;

use std::sync::{Arc, Mutex};

use tauri::Manager;

use application::{AssetApplicationService, JobApplicationService, ProjectApplicationService};
use commands::{
    archive_project, cancel_job, convert_pixel_art, create_job, create_project, delete_asset,
    delete_project, generate_audio, generate_image, generate_tile, get_asset, get_job, get_project,
    import_asset, list_assets, list_jobs, list_projects, open_project, register_asset,
    remove_background, rename_project, synthesize_speech,
};
use model_config::{
    list_model_profiles, create_model_profile, update_model_profile, delete_model_profile,
    list_routing_rules, set_routing_rule, list_prompt_templates, create_prompt_template,
    delete_prompt_template, list_providers, get_provider, test_provider_connection,
    get_credential_status, set_credential, delete_credential, set_provider_enabled,
    ModelConfigService, register_builtin_providers, SqliteModelConfigRepository,
    KeychainCredentialStore,
};
use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::ModelRouter;
use repositories::{SqliteAssetRepository, SqliteJobRepository, SqliteProjectRepository};
use state::AppState;
use workers::{AudioGenWorker, ImageGenWorker, ImageProcessWorker, TileWorker, WorkerRunner};

/// Attempts to create a keychain credential store.
///
/// Returns Ok if the keychain is accessible, Err otherwise.
/// The error contains a message describing why keychain is unavailable.
fn try_create_keychain_store() -> Result<KeychainCredentialStore, String> {
    let store = KeychainCredentialStore::new();
    // Probe the keychain by attempting to list credentials.
    // This will fail if the keyring service is not available.
    match store.list() {
        Ok(_) => Ok(store),
        Err(e) => Err(format!("keychain probe failed: {}", e)),
    }
}

/// Initializes the Tauri application with all plugins and state.
pub fn run_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_dir = app.path().app_data_dir().map_err(|e| {
                format!("Failed to resolve app data directory: {}", e)
            })?;

            std::fs::create_dir_all(&app_dir).map_err(|e| {
                format!("Failed to create app data directory: {}", e)
            })?;

            let db_path = app_dir.join("artifex.db");

            // Initialize database pool synchronously using the current tokio runtime
            let pool = tokio::runtime::Handle::current()
                .block_on(db::init_db_pool(&db_path))
                .map_err(|e| format!("Failed to initialize database: {}", e))?;

            // Create repositories
            let project_repo = Arc::new(SqliteProjectRepository::new(pool.clone()));
            let job_repo = Arc::new(SqliteJobRepository::new(pool.clone()));
            let asset_repo = Arc::new(SqliteAssetRepository::new(pool.clone()));

            // Create application services wrapping repositories
            let project_service = Arc::new(ProjectApplicationService::new(project_repo.clone()));
            let job_service = Arc::new(JobApplicationService::new(job_repo.clone()));
            let asset_service = Arc::new(AssetApplicationService::new(asset_repo.clone()));

            // Create model config infrastructure
            let model_config_repo = Arc::new(SqliteModelConfigRepository::new(pool.clone()));

            // Create provider registry and register built-in providers
            let provider_registry = Arc::new(artifex_model_config::ProviderRegistry::new());
            register_builtin_providers(&provider_registry);

            // Create credential store with graceful degradation.
            // Try KeychainCredentialStore first; if keyring is unavailable, fall back to in-memory.
            let credential_store: Arc<dyn CredentialStore> = match try_create_keychain_store() {
                Ok(store) => {
                    tracing::info!("Using keychain credential store");
                    Arc::new(store)
                }
                Err(e) => {
                    tracing::warn!("Keychain unavailable ({:?}), falling back to in-memory credential store", e);
                    Arc::new(artifex_model_config::credential_store::InMemoryCredentialStore::new())
                }
            };

            // Create model router (uses async ModelConfigRepository trait impl on SqliteModelConfigRepository)
            let model_router = Arc::new(ModelRouter::new(
                provider_registry.clone(),
                model_config_repo.clone(),
                credential_store.clone(),
            ));

            // Create model config service
            let model_config_service = Arc::new(ModelConfigService::new(
                model_config_repo.clone(),
                provider_registry.clone(),
                credential_store.clone(),
            ));

            // Seed default model profiles and routing rules
            tokio::runtime::Handle::current().block_on(async {
                if let Err(e) = model_config_service.seed_defaults().await {
                    tracing::warn!("Failed to seed default model config: {}", e);
                }
            });

            // Create worker infrastructure
            let assets_dir = app_dir.join("artifex-assets").to_string_lossy().to_string();
            let image_worker = Arc::new(ImageGenWorker::new(
                model_router.clone(),
                credential_store.clone(),
                assets_dir.clone(),
            ));
            let image_process_worker = Arc::new(ImageProcessWorker::new(
                model_router.clone(),
                credential_store.clone(),
                assets_dir.clone(),
            ));
            let tile_worker = Arc::new(TileWorker::new(
                model_router.clone(),
                credential_store.clone(),
                assets_dir.clone(),
            ));
            let audio_worker = Arc::new(AudioGenWorker::new(
                model_router.clone(),
                credential_store.clone(),
                assets_dir.clone(),
            ));
            let worker_runner = WorkerRunner::with_app_handle(
                vec![image_worker, image_process_worker, tile_worker, audio_worker],
                job_repo.clone(),
                asset_service.clone(),
                app.handle().clone(),
            );

            // Clone the runner handle before moving app_state
            let runner_handle = Arc::new(Mutex::new(Some(worker_runner)));

            // Set application state
            let app_state = AppState::new(
                project_service,
                job_service,
                asset_service,
                Mutex::new(None),
                runner_handle.clone(),
                model_config_service,
            );
            app.manage(app_state);

            // Spawn worker runner in background
            tokio::spawn(async move {
                let runner = {
                    let mut guard = runner_handle.lock()
                        .expect("runner_handle lock poisoned");
                    guard.take()
                };
                if let Some(r) = runner {
                    r.run().await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_projects,
            create_project,
            get_project,
            open_project,
            rename_project,
            archive_project,
            delete_project,
            create_job,
            list_jobs,
            get_job,
            cancel_job,
            generate_image,
            generate_audio,
            synthesize_speech,
            list_assets,
            get_asset,
            delete_asset,
            import_asset,
            register_asset,
            remove_background,
            convert_pixel_art,
            generate_tile,
            // Model config commands
            list_providers,
            get_provider,
            test_provider_connection,
            set_provider_enabled,
            list_model_profiles,
            create_model_profile,
            update_model_profile,
            delete_model_profile,
            list_routing_rules,
            set_routing_rule,
            list_prompt_templates,
            create_prompt_template,
            delete_prompt_template,
            get_credential_status,
            set_credential,
            delete_credential,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}