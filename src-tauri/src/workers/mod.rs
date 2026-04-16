//! Worker module.
//!
//! Contains the worker traits and implementations for processing jobs.

pub mod traits;
pub mod image_gen_provider;
pub mod image_gen_worker;
pub mod audio_gen_worker;
pub mod runner;

pub use traits::{JobResult, JobWorker};
pub use image_gen_provider::{ImageGenParams, ImageGenProvider, ImageGenResult};
pub use image_gen_worker::ImageGenWorker;
pub use audio_gen_worker::AudioGenWorker;
pub use runner::WorkerRunner;