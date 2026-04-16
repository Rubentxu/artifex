//! IPC command handlers.

pub mod assets;
pub mod audio_gen;
pub mod image_gen;
pub mod jobs;
pub mod projects;

pub use assets::{delete_asset, get_asset, import_asset, list_assets, register_asset};
pub use audio_gen::{generate_audio, synthesize_speech};
pub use image_gen::generate_image;
pub use jobs::{cancel_job, create_job, get_job, list_jobs};
pub use projects::{
    archive_project, create_project, delete_project, get_project, list_projects, open_project,
    rename_project,
};
