//! IPC command handlers.

pub mod animations;
pub mod atlas;
pub mod assets;
pub mod audio_gen;
pub mod code_gen;
pub mod convert_pixel_art;
pub mod generate_material;
pub mod generate_sprite_sheet;
pub mod generate_tile;
pub mod image_gen;
pub mod inpaint_image;
pub mod jobs;
pub mod list_code_templates;
pub mod outpaint_image;
pub mod projects;
pub mod publish;
pub mod quick_sprites;
pub mod remove_background;
pub mod renderer_3d;
pub mod slice_sprite_sheet;
pub mod texture;
pub mod video_gen;

// Re-export identity commands from identity module
pub use crate::identity::commands::{check_quota, get_current_user, get_usage, set_tier, update_profile};

pub use animations::{create_animation, delete_animation, export_animation, get_animation, list_animations, update_animation};
pub use atlas::pack_atlas;
pub use assets::{delete_asset, get_asset, import_asset, list_assets, register_asset};
pub use audio_gen::{generate_audio, synthesize_speech};
pub use code_gen::{generate_code, start_code_agent};
pub use convert_pixel_art::convert_pixel_art;
pub use generate_material::generate_material;
pub use generate_sprite_sheet::generate_sprite_sheet;
pub use generate_tile::generate_tile;
pub use image_gen::generate_image;
pub use inpaint_image::inpaint_image;
pub use jobs::{cancel_job, create_job, get_job, list_jobs};
pub use list_code_templates::list_code_templates;
pub use outpaint_image::outpaint_image;
pub use projects::{
    archive_project, create_project, delete_project, get_project, list_projects, open_project,
    rename_project,
};
pub use publish::{export_project, open_itch_io};
pub use quick_sprites::generate_quick_sprites;
pub use remove_background::remove_background;
pub use renderer_3d::render_3d_to_sprites;
pub use slice_sprite_sheet::slice_sprite_sheet;
pub use texture::generate_seamless_texture;
pub use video_gen::generate_video;
