//! Worker infrastructure setup.

use std::sync::Arc;

use artifex_model_config::credential_store::CredentialStore;
use artifex_model_config::ModelRouter;

use crate::application::AssetApplicationService;
use crate::workers::{
    AnimationExportWorker, AtlasPackWorker, AudioGenWorker, CodeWorker, ImageGenWorker,
    ImageProcessWorker, MaterialWorker, QuickSpritesWorker, Renderer3dWorker, SliceWorker,
    SpriteWorker, TileWorker, SeamlessTextureWorker, VideoGenWorker, WorkerRunner,
};
use crate::repositories::SqliteJobRepository;
use tauri::AppHandle;

/// Creates worker instances and the worker runner.
pub fn create_workers(
    model_router: Arc<ModelRouter>,
    credential_store: Arc<dyn CredentialStore>,
    assets_dir: String,
    job_repo: Arc<SqliteJobRepository>,
    asset_service: Arc<AssetApplicationService>,
    app_handle: AppHandle,
) -> WorkerRunner {
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
    let seamless_worker = Arc::new(SeamlessTextureWorker::new(
        model_router.clone(),
        credential_store.clone(),
        assets_dir.clone(),
    ));
    let audio_worker = Arc::new(AudioGenWorker::new(
        model_router.clone(),
        credential_store.clone(),
        assets_dir.clone(),
    ));
    let sprite_worker = Arc::new(SpriteWorker::new(assets_dir.clone()));
    let slice_worker = Arc::new(SliceWorker::new(assets_dir.clone()));
    let code_worker = Arc::new(CodeWorker::new(
        assets_dir.clone(),
        model_router.clone(),
        credential_store.clone(),
    ));
    let material_worker = Arc::new(MaterialWorker::new(
        model_router.clone(),
        credential_store.clone(),
        assets_dir.clone(),
    ));
    let animation_export_worker = Arc::new(AnimationExportWorker::new(assets_dir.clone()));
    let atlas_pack_worker = Arc::new(AtlasPackWorker::new(assets_dir.clone()));
    let video_worker = Arc::new(VideoGenWorker::new(
        model_router.clone(),
        credential_store.clone(),
        assets_dir.clone(),
    ));
    let quick_sprites_worker = Arc::new(QuickSpritesWorker::new(
        model_router.clone(),
        credential_store.clone(),
        assets_dir.clone(),
    ));
    let renderer_3d_worker = Arc::new(Renderer3dWorker::new(assets_dir.clone()));

    WorkerRunner::with_app_handle(
        vec![
            image_worker,
            image_process_worker,
            tile_worker,
            seamless_worker,
            audio_worker,
            sprite_worker,
            slice_worker,
            code_worker,
            material_worker,
            animation_export_worker,
            atlas_pack_worker,
            video_worker,
            quick_sprites_worker,
            renderer_3d_worker,
        ],
        job_repo,
        asset_service,
        app_handle,
    )
}
