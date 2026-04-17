//! Worker module.
//!
//! Contains the worker traits and implementations for processing jobs.

pub mod traits;
pub mod atlas_pack_worker;
pub mod code_worker;
pub mod image_gen_provider;
pub mod image_gen_worker;
pub mod image_process_worker;
pub mod material_worker;
pub mod sprite_worker;
pub mod slice_worker;
pub mod tile_worker;
pub mod audio_gen_worker;
pub mod animation_export_worker;
pub mod seamless_texture_worker;
pub mod runner;

pub use traits::{JobFuture, JobResult, JobWorker};
pub use atlas_pack_worker::AtlasPackWorker;
pub use code_worker::{CodeWorker, CodeGenOperation, CodeFile};
pub use image_gen_provider::{ImageGenParams, ImageGenProvider, ImageGenResult};
pub use image_gen_worker::ImageGenWorker;
pub use image_process_worker::ImageProcessWorker;
pub use material_worker::MaterialWorker;
pub use sprite_worker::SpriteWorker;
pub use slice_worker::SliceWorker;
pub use tile_worker::TileWorker;
pub use audio_gen_worker::AudioGenWorker;
pub use animation_export_worker::AnimationExportWorker;
pub use seamless_texture_worker::SeamlessTextureWorker;
pub use runner::WorkerRunner;