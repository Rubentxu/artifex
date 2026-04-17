//! IPC command handlers.

pub mod assets;
pub mod audio_gen;
pub mod convert_pixel_art;
pub mod generate_sprite_sheet;
pub mod generate_tile;
pub mod image_gen;
pub mod jobs;
pub mod projects;
pub mod remove_background;
pub mod slice_sprite_sheet;

pub use assets::{delete_asset, get_asset, import_asset, list_assets, register_asset};
pub use audio_gen::{generate_audio, synthesize_speech};
pub use convert_pixel_art::convert_pixel_art;
pub use generate_sprite_sheet::generate_sprite_sheet;
pub use generate_tile::generate_tile;
pub use image_gen::generate_image;
pub use jobs::{cancel_job, create_job, get_job, list_jobs};
pub use projects::{
    archive_project, create_project, delete_project, get_project, list_projects, open_project,
    rename_project,
};
pub use remove_background::remove_background;
pub use slice_sprite_sheet::slice_sprite_sheet;
