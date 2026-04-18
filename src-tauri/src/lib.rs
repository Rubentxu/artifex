//! Artifex library crate.
//!
//! This lib.rs contains all the modules including command definitions.

pub mod application;
pub mod bootstrap;
pub mod commands;
pub mod db;
pub mod dto;
pub mod model_config;
pub mod repositories;
pub mod state;
pub use state::AppState;
pub mod workers;

use tauri::Manager;

use commands::{
    archive_project, cancel_job, convert_pixel_art, create_animation, create_job, create_project,
    delete_animation, delete_asset, delete_project, export_animation, export_project, generate_audio,
    generate_code, generate_image, generate_material, generate_quick_sprites, generate_seamless_texture, generate_sprite_sheet, generate_tile, generate_video,
    get_animation, get_asset, get_job, get_project, import_asset, inpaint_image, list_animations,
    list_assets, list_code_templates, list_jobs, list_projects, open_itch_io, open_project, outpaint_image,
    pack_atlas, register_asset, remove_background, rename_project, render_3d_to_sprites, slice_sprite_sheet, synthesize_speech,
    update_animation,
};
use model_config::{
    list_model_profiles, create_model_profile, update_model_profile, delete_model_profile,
    list_routing_rules, set_routing_rule, list_prompt_templates, create_prompt_template,
    delete_prompt_template, list_providers, get_provider, test_provider_connection,
    get_credential_status, set_credential, delete_credential, set_provider_enabled,
};
use bootstrap::app::{setup_app, spawn_worker_runner};

/// Initializes the Tauri application with all plugins and state.
pub fn run_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_state = setup_app(app)?;
            spawn_worker_runner(&app_state);
            app.manage(app_state);
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
            // Animation commands
            create_animation,
            list_animations,
            get_animation,
            update_animation,
            delete_animation,
            export_animation,
            remove_background,
            inpaint_image,
            outpaint_image,
            convert_pixel_art,
            generate_tile,
            generate_seamless_texture,
            generate_sprite_sheet,
            generate_quick_sprites,
            render_3d_to_sprites,
            generate_video,
            slice_sprite_sheet,
            pack_atlas,
            generate_code,
            generate_material,
            list_code_templates,
            // Publishing commands
            export_project,
            open_itch_io,
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
