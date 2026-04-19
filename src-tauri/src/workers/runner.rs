//! Worker runner that polls for jobs and dispatches to workers.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, timeout};

use artifex_job_queue::{Job, JobRepository, JobStatus};
use artifex_shared_kernel::AppError;

use crate::application::AssetApplicationService;

use super::traits::{JobResult, JobWorker, WorkerCategory};

/// Timeout duration for async worker jobs (5 minutes).
const ASYNC_TIMEOUT: Duration = Duration::from_secs(5 * 60);
/// Timeout duration for CPU-intensive worker jobs (10 minutes).
const CPU_TIMEOUT: Duration = Duration::from_secs(10 * 60);
/// Maximum number of retry attempts before marking a job as permanently failed.
const MAX_RETRIES: u8 = 3;

/// Event payload for job progress updates.
#[derive(Clone, Serialize)]
pub struct JobProgressEvent {
    pub job_id: String,
    pub progress_percent: u8,
    pub progress_message: String,
    /// Current attempt number (0-based).
    #[serde(default)]
    pub attempt: u8,
    /// Whether the job has exhausted all retry attempts.
    #[serde(default)]
    pub max_attempts_reached: bool,
    /// Whether the job failed due to a timeout.
    #[serde(default)]
    pub timeout: bool,
}

/// Event payload for job completion.
#[derive(Clone, Serialize)]
pub struct JobCompletedEvent {
    pub job_id: String,
    pub asset_ids: Vec<String>,
}

/// Event payload for job failure.
#[derive(Clone, Serialize)]
pub struct JobFailedEvent {
    pub job_id: String,
    pub error_message: String,
}

/// Runner that polls for pending jobs and dispatches to appropriate workers.
pub struct WorkerRunner {
    /// Registered workers that can process jobs.
    workers: Vec<Arc<dyn JobWorker>>,
    /// Job repository for accessing job queue.
    job_repo: Arc<dyn JobRepository>,
    /// Asset service for registering completed assets.
    asset_service: Arc<AssetApplicationService>,
    /// Tauri app handle for emitting events.
    app_handle: Option<AppHandle>,
    /// Shutdown flag.
    shutdown: Arc<AtomicBool>,
    /// Poll interval between job queue checks.
    poll_interval: Duration,
}

impl WorkerRunner {
    /// Creates a new WorkerRunner.
    pub fn new(
        workers: Vec<Arc<dyn JobWorker>>,
        job_repo: Arc<dyn JobRepository>,
        asset_service: Arc<AssetApplicationService>,
    ) -> Self {
        Self {
            workers,
            job_repo,
            asset_service,
            app_handle: None,
            shutdown: Arc::new(AtomicBool::new(false)),
            poll_interval: Duration::from_millis(500),
        }
    }

    /// Creates a new WorkerRunner with an app handle for event emission.
    pub fn with_app_handle(
        workers: Vec<Arc<dyn JobWorker>>,
        job_repo: Arc<dyn JobRepository>,
        asset_service: Arc<AssetApplicationService>,
        app_handle: AppHandle,
    ) -> Self {
        Self {
            workers,
            job_repo,
            asset_service,
            app_handle: Some(app_handle),
            shutdown: Arc::new(AtomicBool::new(false)),
            poll_interval: Duration::from_millis(500),
        }
    }

    /// Creates a new WorkerRunner with a custom poll interval.
    pub fn with_poll_interval(
        workers: Vec<Arc<dyn JobWorker>>,
        job_repo: Arc<dyn JobRepository>,
        asset_service: Arc<AssetApplicationService>,
        poll_interval: Duration,
    ) -> Self {
        Self {
            workers,
            job_repo,
            asset_service,
            app_handle: None,
            shutdown: Arc::new(AtomicBool::new(false)),
            poll_interval,
        }
    }

    /// Signals the runner to shut down.
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::SeqCst);
    }

    /// Returns true if shutdown has been requested.
    pub fn is_shutting_down(&self) -> bool {
        self.shutdown.load(Ordering::SeqCst)
    }

    /// Main run loop. Polls for pending jobs and dispatches to workers.
    pub async fn run(&self) {
        tracing::info!("WorkerRunner started");

        while !self.is_shutting_down() {
            // Check for shutdown before polling
            if self.is_shutting_down() {
                break;
            }

            // Poll for pending jobs across all projects
            if let Err(e) = self.process_pending_jobs().await {
                tracing::error!("Error processing pending jobs: {}", e);
            }

            // Sleep before next poll
            sleep(self.poll_interval).await;
        }

        tracing::info!("WorkerRunner stopped");
    }

    /// Processes all pending jobs in the queue.
    async fn process_pending_jobs(&self) -> Result<(), AppError> {
        let pending_jobs = self.fetch_pending_jobs().await?;

        // Clone app_handle for use in spawned tasks
        let app_handle = self.app_handle.clone();

        for job in pending_jobs {
            // Check shutdown flag before processing each job
            if self.is_shutting_down() {
                break;
            }

            // Spawn a task to process this job
            let workers = self.workers.clone();
            let job_repo = self.job_repo.clone();
            let asset_service = self.asset_service.clone();
            let app_handle_clone = app_handle.clone();
            let job_id = job.id;

            tokio::spawn(async move {
                if let Err(e) = Self::process_single_job(workers, job_repo, asset_service, app_handle_clone.as_ref(), job).await {
                    tracing::error!("Job {} processing failed: {}", job_id.into_uuid(), e);
                }
            });
        }

        Ok(())
    }

    /// Fetches pending jobs that need processing.
    async fn fetch_pending_jobs(&self) -> Result<Vec<Job>, AppError> {
        self.job_repo
            .list_all_by_status(JobStatus::Pending)
            .await
    }

    /// Processes a single job with panic isolation, timeout enforcement, and retry logic.
    async fn process_single_job(
        workers: Vec<Arc<dyn JobWorker>>,
        job_repo: Arc<dyn JobRepository>,
        asset_service: Arc<AssetApplicationService>,
        app_handle: Option<&tauri::AppHandle>,
        job: Job,
    ) -> Result<(), AppError> {
        let job_id_str = job.id.into_uuid().to_string();
        let job_id = job.id;

        // Skip cancelled jobs
        if job.status == JobStatus::Cancelled {
            tracing::debug!("Skipping cancelled job {}", job_id_str);
            return Ok(());
        }

        // Skip non-pending jobs
        if job.status != JobStatus::Pending {
            return Ok(());
        }

        // Find a worker that can handle this job
        let worker = workers
            .iter()
            .find(|w| w.can_handle(&job.job_type))
            .ok_or_else(|| {
                AppError::validation(format!(
                    "No worker found for job type '{}'",
                    job.job_type
                ))
            })?;

        let category = worker.category();
        tracing::info!(
            "Processing job {} of type '{}' with category {:?}",
            job_id_str,
            job.job_type,
            category
        );

        // Update status to Running (this also sets started_at in the repository)
        if let Err(e) = job_repo.update_status(&job.id, JobStatus::Running).await {
            tracing::error!("Failed to update job {} status to Running: {}", job_id_str, e);
            return Err(e);
        }

        // Helper: emit progress event and persist to DB at each phase
        let emit_progress = |percent: u8,
                             message: &'static str,
                             attempt: u8,
                             max_attempts_reached: bool,
                             is_timeout: bool| {
            let job_id = job.id;
            let app_handle = app_handle.cloned();
            let job_repo = job_repo.clone();
            async move {
                // Emit event
                if let Some(handle) = app_handle {
                    let progress_event = JobProgressEvent {
                        job_id: job_id.into_uuid().to_string(),
                        progress_percent: percent,
                        progress_message: message.to_string(),
                        attempt,
                        max_attempts_reached,
                        timeout: is_timeout,
                    };
                    let _ = handle.emit("job-progress", progress_event);
                }
                // Persist to DB (best effort — don't fail the job on persistence error)
                let _ = job_repo
                    .update_progress(&job_id, percent, Some(message))
                    .await;
            }
        };

        // Phase 1: Queued (10%) — before dispatch
        emit_progress(10, "Job queued", job.attempt, false, false).await;

        // Phase 2: Calling provider (30%) — just before worker.process()
        emit_progress(30, "Calling provider", job.attempt, false, false).await;

        // Determine timeout based on worker category
        let timeout_duration = match category {
            WorkerCategory::Async => ASYNC_TIMEOUT,
            WorkerCategory::CpuIntensive => CPU_TIMEOUT,
        };

        // Dispatch the job with panic isolation and timeout.
        //
        // Strategy:
        // - For both async and CPU-intensive workers, we use tokio::spawn to run the worker's
        //   process() future.
        // - For CPU-intensive workers, the worker's async process() may internally use
        //   spawn_blocking or blocking libraries (like wgpu), so the extended timeout (10 min)
        //   allows for longer-running CPU-bound operations.
        // - Panic handling: when an async task panics, tokio's JoinHandle will return
        //   Err(JoinError) with is_panic() == true. We catch this and log the panic.
        // - If the timeout fires, the spawned task is aborted and we return a timeout error.

        let worker = worker.clone();
        let job_for_spawn = job.clone();
        let job_id_for_completion = job.id; // Capture id before move

        let join_handle = tokio::spawn(async move {
            worker.process(&job_for_spawn).await
        });

        // Apply timeout to the spawned task
        let process_result: Result<Result<JobResult, String>, tokio::time::error::Elapsed> =
            tokio::time::timeout(timeout_duration, join_handle)
                .await
                .map(|join_result| {
                    match join_result {
                        Ok(Ok(job_result)) => Ok(job_result),
                        Ok(Err(app_error)) => {
                            // Worker returned an error (not a panic)
                            Err(format!("Worker error: {}", app_error))
                        }
                        Err(join_error) if join_error.is_panic() => {
                            // Task panicked
                            Err("Worker process panicked".to_string())
                        }
                        Err(join_error) => {
                            // Join error (task was cancelled, etc.)
                            Err(format!("Join error: {}", join_error))
                        }
                    }
                });

        // Phase 3: Receiving result (70%) — after worker.process() returns
        emit_progress(70, "Receiving result", job.attempt, false, false).await;

        // Handle the result: success, panic, or timeout
        let job_result: JobResult = match process_result {
            Ok(Ok(inner_result)) => {
                // Success
                inner_result
            }
            Ok(Err(panic_msg)) => {
                // Panic - the panic message is already captured as a String
                tracing::error!("Job {} panicked: {}", job_id_str, panic_msg);

                // Emit job-failed event
                if let Some(handle) = app_handle {
                    let failed_event = JobFailedEvent {
                        job_id: job_id_str.clone(),
                        error_message: format!("Panic: {}", panic_msg),
                    };
                    let _ = handle.emit("job-failed", failed_event);
                }

                // Update failure status and get current attempt
                if let Err(update_err) = job_repo
                    .update_failure(&job_id, &format!("Panic: {}", panic_msg))
                    .await
                {
                    tracing::error!(
                        "Failed to update job {} failure status: {}",
                        job_id_str,
                        update_err
                    );
                }

                // Re-fetch job to get updated attempt count
                let updated_job = job_repo.find_by_id(&job_id).await?.unwrap_or_else(|| job.clone());
                let current_attempt = updated_job.attempt;

                // Check if we should retry
                if current_attempt < MAX_RETRIES {
                    // Retry with exponential backoff
                    let backoff_secs = 2u64.pow(current_attempt as u32);
                    tracing::info!(
                        "Job {} will retry (attempt {}/{}) after {}s backoff",
                        job_id_str,
                        current_attempt + 1,
                        MAX_RETRIES,
                        backoff_secs
                    );
                    emit_progress(30, "Retrying", current_attempt + 1, false, false).await;
                    sleep(Duration::from_secs(backoff_secs)).await;

                    // Re-enqueue as pending
                    if let Err(e) = job_repo.update_status(&job_id, JobStatus::Pending).await {
                        tracing::error!("Failed to re-enqueue job {}: {}", job_id_str, e);
                    }
                } else {
                    // Max retries reached - permanent failure
                    tracing::error!(
                        "Job {} permanently failed after {} attempts",
                        job_id_str,
                        MAX_RETRIES
                    );
                    emit_progress(100, "Failed (max retries)", current_attempt, true, false).await;
                }
                return Ok(());
            }
            Err(_timeout_err) => {
                // Timeout
                tracing::error!("Job {} timed out", job_id_str);

                // Emit job-failed event
                if let Some(handle) = app_handle {
                    let failed_event = JobFailedEvent {
                        job_id: job_id_str.clone(),
                        error_message: "Job timed out".to_string(),
                    };
                    let _ = handle.emit("job-failed", failed_event);
                }

                // Update failure status
                if let Err(update_err) = job_repo
                    .update_failure(&job_id, "Job timed out")
                    .await
                {
                    tracing::error!(
                        "Failed to update job {} failure status: {}",
                        job_id_str,
                        update_err
                    );
                }

                // Re-fetch job to get updated attempt count
                let updated_job = job_repo.find_by_id(&job_id).await?.unwrap_or_else(|| job.clone());
                let current_attempt = updated_job.attempt;

                // Check if we should retry
                if current_attempt < MAX_RETRIES {
                    // Retry with exponential backoff
                    let backoff_secs = 2u64.pow(current_attempt as u32);
                    tracing::info!(
                        "Job {} will retry (attempt {}/{}) after {}s backoff",
                        job_id_str,
                        current_attempt + 1,
                        MAX_RETRIES,
                        backoff_secs
                    );
                    emit_progress(30, "Retrying", current_attempt + 1, false, true).await;
                    sleep(Duration::from_secs(backoff_secs)).await;

                    // Re-enqueue as pending
                    if let Err(e) = job_repo.update_status(&job_id, JobStatus::Pending).await {
                        tracing::error!("Failed to re-enqueue job {}: {}", job_id_str, e);
                    }
                } else {
                    // Max retries reached - permanent failure
                    tracing::error!(
                        "Job {} permanently failed after {} attempts (timeout)",
                        job_id_str,
                        MAX_RETRIES
                    );
                    emit_progress(100, "Failed (max retries)", current_attempt, true, true).await;
                }
                return Ok(());
            }
        };

        // Success path - job completed without panic or timeout
        let result = job_result;

        // Phase 4: Saving asset (90%) — before asset registration
        emit_progress(90, "Saving asset", job.attempt, false, false).await;

        // Register asset(s) for completed job output
        let mut asset_ids = Vec::new();
        if !result.output_files.is_empty() {
            // Determine asset kind based on job type
            let asset_kind = match job.job_type.as_str() {
                "audio_generate" => "audio",
                "tts_synthesize" => "voice",
                "image_generate"
                | "image_remove_background"
                | "pixel_art_convert"
                | "image_inpaint"
                | "image_outpaint"
                | "sprite_slice" => "image",
                "tile_generate" | "seamless_texture" => "tileset",
                "sprite_generate" | "pack_atlas" | "quick_sprites" | "render_3d" => "sprite",
                "material_generate" => "material",
                "code_generate" => "code",
                "animation_export" => "animation",
                "video_generate" => "video",
                _ => "unknown",
            };

            // For sprite_generate: only register the atlas PNG (first output file)
            // Manifest and Aseprite JSON paths are stored in metadata only
            let output_files_to_register = if job.job_type == "sprite_generate" {
                result.output_files.iter().take(1).collect::<Vec<_>>()
            } else {
                result.output_files.iter().collect::<Vec<_>>()
            };

            for output_file in output_files_to_register {
                let project_id = job.project_id.into_uuid().to_string();
                let asset_name = output_file
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| format!("generated_{}.bin", asset_kind));

                // For sprite_generate: include manifest/aseprite paths in metadata
                // For other job types: use standard metadata
                let metadata = Some(result.metadata.clone());
                let file_path = output_file.to_string_lossy().to_string();

                // Extract source_asset_id from job operation for lineage tracking
                let source_asset_id = job.operation
                    .get("source_asset_id")
                    .and_then(|v| v.as_str());

                let asset_service_clone = asset_service.clone();
                match asset_service_clone
                    .register_asset_with_lineage(
                        &project_id,
                        &asset_name,
                        asset_kind,
                        &file_path,
                        metadata,
                        source_asset_id,
                    )
                    .await
                {
                    Ok(asset) => {
                        tracing::info!(
                            "Registered asset {} for job output",
                            asset.id.into_uuid()
                        );
                        asset_ids.push(asset.id.into_uuid().to_string());
                    }
                    Err(e) => {
                        tracing::error!("Failed to register asset for job output: {}", e);
                    }
                }
            }
        }

        // Phase 5: Completed (100%) — after asset registration
        let _ = job_repo
            .update_progress(&job.id, 100, Some("Completed"))
            .await;
        emit_progress(100, "Complete", job.attempt, false, false).await;
        let _ = job_repo.mark_completed(&job.id).await;

        // Emit job-completed event
        if let Some(handle) = app_handle {
            let completed_event = JobCompletedEvent {
                job_id: job_id_str.clone(),
                asset_ids,
            };
            let _ = handle.emit("job-completed", completed_event);
        }

        tracing::info!(
            "Job {} completed successfully, output files: {:?}",
            job_id_str,
            result.output_files
        );

        Ok(())
    }

    /// Finds a worker that can handle the given job type.
    pub fn find_worker(&self, job_type: &str) -> Option<&Arc<dyn JobWorker>> {
        self.workers.iter().find(|w| w.can_handle(job_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;
    use artifex_shared_kernel::{JobId, ProjectId};
    use crate::application::AssetApplicationService;
    use super::super::traits::{JobFuture, JobResult, WorkerCategory};

    /// A mock job repository for testing.
    struct MockJobRepository {
        pending_jobs: Vec<Job>,
        status_updates: Arc<AtomicUsize>,
        progress_updates: Arc<AtomicUsize>,
    }

    impl MockJobRepository {
        fn new(pending_jobs: Vec<Job>) -> Self {
            Self {
                pending_jobs,
                status_updates: Arc::new(AtomicUsize::new(0)),
                progress_updates: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    #[async_trait::async_trait]
    impl JobRepository for MockJobRepository {
        async fn create(&self, _job: &Job) -> Result<(), artifex_shared_kernel::ArtifexError> {
            Ok(())
        }

        async fn find_by_id(&self, _id: &JobId) -> Result<Option<Job>, artifex_shared_kernel::ArtifexError> {
            Ok(None)
        }

        async fn list_by_project(&self, _project_id: &ProjectId) -> Result<Vec<Job>, artifex_shared_kernel::ArtifexError> {
            Ok(vec![])
        }

        async fn list_by_status(&self, _project_id: &ProjectId, _status: JobStatus) -> Result<Vec<Job>, artifex_shared_kernel::ArtifexError> {
            Ok(self.pending_jobs.clone())
        }

        async fn list_all_by_status(&self, _status: JobStatus) -> Result<Vec<Job>, artifex_shared_kernel::ArtifexError> {
            Ok(self.pending_jobs.clone())
        }

        async fn update_status(&self, _id: &JobId, _status: JobStatus) -> Result<(), artifex_shared_kernel::ArtifexError> {
            self.status_updates.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn update_progress(&self, _id: &JobId, _percent: u8, _message: Option<&str>) -> Result<(), artifex_shared_kernel::ArtifexError> {
            self.progress_updates.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }

        async fn update_failure(&self, _id: &JobId, _error_message: &str) -> Result<(), artifex_shared_kernel::ArtifexError> {
            Ok(())
        }

        async fn mark_completed(&self, _id: &JobId) -> Result<(), artifex_shared_kernel::ArtifexError> {
            Ok(())
        }
    }

    /// A mock asset repository for testing.
    struct MockAssetRepository;

    #[async_trait::async_trait]
    impl artifex_asset_management::AssetRepository for MockAssetRepository {
        async fn create(&self, _asset: &artifex_asset_management::Asset) -> Result<artifex_asset_management::Asset, artifex_shared_kernel::ArtifexError> {
            Ok(_asset.clone())
        }

        async fn find_by_id(&self, _id: &artifex_shared_kernel::AssetId) -> Result<Option<artifex_asset_management::Asset>, artifex_shared_kernel::ArtifexError> {
            Ok(None)
        }

        async fn find_by_project(&self, _project_id: &artifex_shared_kernel::ProjectId) -> Result<Vec<artifex_asset_management::Asset>, artifex_shared_kernel::ArtifexError> {
            Ok(vec![])
        }

        async fn find_by_kind(&self, _project_id: &artifex_shared_kernel::ProjectId, _kind: &artifex_asset_management::AssetKind) -> Result<Vec<artifex_asset_management::Asset>, artifex_shared_kernel::ArtifexError> {
            Ok(vec![])
        }

        async fn find_by_tag(&self, _project_id: &artifex_shared_kernel::ProjectId, _tag: &str) -> Result<Vec<artifex_asset_management::Asset>, artifex_shared_kernel::ArtifexError> {
            Ok(vec![])
        }

        async fn find_by_collection(&self, _project_id: &artifex_shared_kernel::ProjectId, _collection_id: &str) -> Result<Vec<artifex_asset_management::Asset>, artifex_shared_kernel::ArtifexError> {
            Ok(vec![])
        }

        async fn update_tags(&self, _id: &artifex_shared_kernel::AssetId, _tags: &[String]) -> Result<(), artifex_shared_kernel::ArtifexError> {
            Ok(())
        }

        async fn update_collection(&self, _id: &artifex_shared_kernel::AssetId, _collection_id: Option<&str>) -> Result<(), artifex_shared_kernel::ArtifexError> {
            Ok(())
        }

        async fn delete(&self, _id: &artifex_shared_kernel::AssetId) -> Result<(), artifex_shared_kernel::ArtifexError> {
            Ok(())
        }
    }

    /// A mock job worker for testing.
    struct MockWorker {
        job_type: String,
        process_count: Arc<AtomicUsize>,
    }

    impl MockWorker {
        fn new(job_type: String) -> Self {
            Self {
                job_type,
                process_count: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl JobWorker for MockWorker {
        fn can_handle(&self, job_type: &str) -> bool {
            job_type == self.job_type
        }

        fn process(&self, _job: &Job) -> JobFuture {
            let count = self.process_count.clone();
            Box::pin(async move {
                count.fetch_add(1, Ordering::SeqCst);
                Ok(JobResult::new(vec![]))
            })
        }
    }

    /// A mock worker that overrides category to CpuIntensive.
    struct MockCpuIntensiveWorker {
        job_type: String,
    }

    impl MockCpuIntensiveWorker {
        fn new(job_type: String) -> Self {
            Self { job_type }
        }
    }

    impl JobWorker for MockCpuIntensiveWorker {
        fn can_handle(&self, job_type: &str) -> bool {
            job_type == self.job_type
        }

        fn category(&self) -> WorkerCategory {
            WorkerCategory::CpuIntensive
        }

        fn process(&self, _job: &Job) -> JobFuture {
            Box::pin(async move {
                Ok(JobResult::new(vec![]))
            })
        }
    }

    /// A mock worker that panics when processed.
    struct MockPanickingWorker {
        job_type: String,
    }

    impl MockPanickingWorker {
        fn new(job_type: String) -> Self {
            Self { job_type }
        }
    }

    impl JobWorker for MockPanickingWorker {
        fn can_handle(&self, job_type: &str) -> bool {
            job_type == self.job_type
        }

        fn process(&self, _job: &Job) -> JobFuture {
            Box::pin(async move {
                panic!("Mock worker panicked as requested");
            })
        }
    }

    /// A mock worker that sleeps longer than the timeout.
    struct MockSlowWorker {
        job_type: String,
        sleep_duration: Duration,
    }

    impl MockSlowWorker {
        fn new(job_type: String, sleep_duration: Duration) -> Self {
            Self {
                job_type,
                sleep_duration,
            }
        }
    }

    impl JobWorker for MockSlowWorker {
        fn can_handle(&self, job_type: &str) -> bool {
            job_type == self.job_type
        }

        fn process(&self, _job: &Job) -> JobFuture {
            let duration = self.sleep_duration;
            Box::pin(async move {
                tokio::time::sleep(duration).await;
                Ok(JobResult::new(vec![]))
            })
        }
    }

    fn make_mock_asset_service() -> Arc<AssetApplicationService> {
        Arc::new(AssetApplicationService::new(Arc::new(MockAssetRepository)))
    }

    #[test]
    fn test_worker_runner_finds_matching_worker() {
        let worker = Arc::new(MockWorker::new("test_job".to_string()));
        let runner = WorkerRunner::new(
            vec![worker],
            Arc::new(MockJobRepository::new(vec![])),
            make_mock_asset_service(),
        );

        assert!(runner.find_worker("test_job").is_some());
        assert!(runner.find_worker("other_job").is_none());
    }

    #[test]
    fn test_shutdown_flag() {
        let runner = WorkerRunner::new(
            vec![],
            Arc::new(MockJobRepository::new(vec![])),
            make_mock_asset_service(),
        );

        assert!(!runner.is_shutting_down());

        runner.shutdown();

        assert!(runner.is_shutting_down());
    }

    #[test]
    fn test_find_worker_returns_first_match() {
        let worker1 = Arc::new(MockWorker::new("test_job".to_string()));
        let worker2 = Arc::new(MockWorker::new("test_job".to_string()));
        let runner = WorkerRunner::new(
            vec![worker1.clone(), worker2],
            Arc::new(MockJobRepository::new(vec![])),
            make_mock_asset_service(),
        );

        let found = runner.find_worker("test_job");
        assert!(found.is_some());
    }

    // =============================================================================
    // WorkerCategory Tests
    // =============================================================================

    #[test]
    fn test_worker_default_category_is_async() {
        let worker = MockWorker::new("test_job".to_string());
        assert_eq!(worker.category(), WorkerCategory::Async);
    }

    #[test]
    fn test_cpu_intensive_worker_category() {
        let worker = MockCpuIntensiveWorker::new("cpu_job".to_string());
        assert_eq!(worker.category(), WorkerCategory::CpuIntensive);
    }

    #[test]
    fn test_worker_runner_finds_cpu_intensive_worker() {
        let worker = Arc::new(MockCpuIntensiveWorker::new("cpu_job".to_string()));
        let runner = WorkerRunner::new(
            vec![worker],
            Arc::new(MockJobRepository::new(vec![])),
            make_mock_asset_service(),
        );

        let found = runner.find_worker("cpu_job");
        assert!(found.is_some());
        assert_eq!(found.unwrap().category(), WorkerCategory::CpuIntensive);
    }
}
