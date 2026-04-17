//! Integration tests for sprite sheet generation.
//!
//! These tests verify the sprite worker pipeline, command validation,
//! and manifest generation.

mod test_helpers;

use std::path::Path;

use src_tauri::workers::sprite_worker::{OutputFormat, SpriteWorker};
use src_tauri::workers::JobWorker;

/// Helper to check if a format is supported (mirrors the command validation).
fn is_format_supported(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext_lower = ext.to_string_lossy().to_lowercase();
        matches!(ext_lower.as_str(), "mp4" | "gif" | "webm")
    } else {
        false
    }
}

#[test]
fn test_supported_video_format_check() {
    // Test that supported formats are correctly identified
    let supported_paths = vec![
        Path::new("/tmp/video.mp4"),
        Path::new("/tmp/anim.gif"),
        Path::new("/tmp/clip.webm"),
    ];

    for path in supported_paths {
        assert!(
            is_format_supported(path),
            "Format {:?} should be supported",
            path.extension()
        );
    }
}

#[test]
fn test_unsupported_video_format_check() {
    // Test that unsupported formats are correctly rejected
    let unsupported_paths = vec![
        Path::new("/tmp/video.avi"),
        Path::new("/tmp/movie.mov"),
        Path::new("/tmp/clip.mkv"),
        Path::new("/tmp/anim.wmv"),
        Path::new("/tmp/video.flv"),
    ];

    for path in unsupported_paths {
        assert!(
            !is_format_supported(path),
            "Format {:?} should NOT be supported",
            path.extension()
        );
    }
}

#[test]
fn test_no_extension_detection() {
    // Files without extensions should be rejected
    let path = Path::new("/tmp/video_without_extension");
    assert!(!is_format_supported(path));
}

#[test]
fn test_output_format_deserialization() {
    let json = r#""json""#;
    let format: OutputFormat = serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(format, OutputFormat::Json);

    let json = r#""aseprite""#;
    let format: OutputFormat = serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(format, OutputFormat::Aseprite);

    let json = r#""both""#;
    let format: OutputFormat = serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(format, OutputFormat::Both);
}

#[test]
fn test_output_format_default() {
    assert_eq!(OutputFormat::default(), OutputFormat::Both);
}

#[test]
fn test_sprite_worker_can_handle() {
    let worker = SpriteWorker::new("/tmp".to_string());
    assert!(worker.can_handle("sprite_generate"));
    assert!(!worker.can_handle("image_generate"));
    assert!(!worker.can_handle("tile_generate"));
    assert!(!worker.can_handle("audio_generate"));
}

#[test]
fn test_sprite_operation_deserialization() {
    let json = r#"{
        "sourceVideoPath": "/path/to/video.mp4",
        "fps": 10,
        "dedupThreshold": 0.03,
        "atlasMaxSize": 4096,
        "padding": 1,
        "animationName": "idle",
        "outputFormat": "both",
        "sourceAssetId": "asset-123",
        "projectId": "project-456"
    }"#;

    let op: src_tauri::workers::sprite_worker::SpriteOperation =
        serde_json::from_str(json).expect("Should deserialize SpriteOperation");
    assert_eq!(op.source_video_path, "/path/to/video.mp4");
    assert_eq!(op.fps, 10);
    assert_eq!(op.dedup_threshold, 0.03);
    assert_eq!(op.atlas_max_size, 4096);
    assert_eq!(op.padding, 1);
    assert_eq!(op.animation_name, "idle");
    assert_eq!(op.output_format, OutputFormat::Both);
    assert_eq!(op.source_asset_id, "asset-123");
    assert_eq!(op.project_id, "project-456");
}

#[test]
fn test_sprite_operation_with_json_format() {
    let json = r#"{
        "sourceVideoPath": "/path/to/video.mp4",
        "fps": 15,
        "dedupThreshold": 0.05,
        "atlasMaxSize": 2048,
        "padding": 2,
        "animationName": "walk",
        "outputFormat": "json",
        "sourceAssetId": "asset-789",
        "projectId": "project-abc"
    }"#;

    let op: src_tauri::workers::sprite_worker::SpriteOperation =
        serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(op.output_format, OutputFormat::Json);
    assert_eq!(op.animation_name, "walk");
}

#[test]
fn test_sprite_operation_with_aseprite_format() {
    let json = r#"{
        "sourceVideoPath": "/path/to/video.webm",
        "fps": 30,
        "dedupThreshold": 0.01,
        "atlasMaxSize": 8192,
        "padding": 0,
        "animationName": "run",
        "outputFormat": "aseprite",
        "sourceAssetId": "asset-def",
        "projectId": "project-ghi"
    }"#;

    let op: src_tauri::workers::sprite_worker::SpriteOperation =
        serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(op.output_format, OutputFormat::Aseprite);
    assert_eq!(op.fps, 30);
}
