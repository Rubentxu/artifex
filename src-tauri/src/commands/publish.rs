//! Publish commands for exporting projects and opening itch.io.

use std::io::Write;
use std::path::Path;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::DialogExt;
use zip::write::SimpleFileOptions;
use zip::ZipWriter;

use crate::dto::ExportProjectResponse;
use crate::state::AppState;

/// Request type for exporting a project.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportProjectRequest {
    pub project_id: String,
    #[serde(default = "default_true")]
    pub include_html_gallery: bool,
    pub output_path: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Asset manifest entry for export.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetManifestEntry {
    pub name: String,
    pub kind: String,
    pub file_path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub file_size: Option<u64>,
    pub metadata: Option<serde_json::Value>,
}

/// Export manifest structure.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportManifest {
    pub version: String,
    pub project_name: String,
    pub exported_at: String,
    pub asset_count: usize,
    pub assets: Vec<AssetManifestEntry>,
}

/// Exports a project as a ZIP file containing all assets.
#[tauri::command]
pub async fn export_project(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    request: ExportProjectRequest,
) -> Result<ExportProjectResponse, String> {
    // Check tier - Pro required for publishing
    let tier = state
        .identity_service
        .get_tier()
        .await
        .map_err(|e| e.to_string())?;
    if !tier.is_pro() {
        return Err("Pro tier required for publishing".to_string());
    }

    let project_id = &request.project_id;

    // Load project info
    let project = state
        .project_service
        .get_project(project_id)
        .await
        .map_err(|e| format!("Failed to load project: {}", e))?;

    // Load all assets for this project
    let assets = state
        .asset_service
        .list_assets(project_id)
        .await
        .map_err(|e| format!("Failed to load assets: {}", e))?;

    // Determine output path
    let output_path = if let Some(ref path) = request.output_path {
        path.clone()
    } else {
        // Use system dialog to pick save location
        let file_path = app
            .dialog()
            .file()
            .set_title("Export Project")
            .set_file_name(&format!("{}.zip", project.name))
            .add_filter("ZIP Archive", &["zip"])
            .blocking_save_file();

        match file_path {
            Some(path) => path.to_string(),
            None => return Err("Export cancelled".to_string()),
        }
    };

    // Create temp directory for export
    let temp_dir = std::env::temp_dir().join(format!("artifex-export-{}", uuid::Uuid::new_v4()));
    let export_dir = temp_dir.join(project.name.to_string());
    std::fs::create_dir_all(&export_dir).map_err(|e| format!("Failed to create temp dir: {}", e))?;

    // Create subdirectories for different asset kinds
    let subdirs = ["images", "audio", "video", "code", "other"];
    for subdir in &subdirs {
        std::fs::create_dir_all(export_dir.join(subdir)).map_err(|e| format!("Failed to create subdir: {}", e))?;
    }

    // Copy assets to appropriate directories
    let mut manifest_assets = Vec::new();
    for asset in &assets {
        if let Some(ref file_path) = asset.file_path {
            let source = Path::new(file_path);
            if source.exists() {
                // Determine destination subdirectory based on asset kind
                let subdir = match asset.kind.as_str() {
                    "image" | "sprite" | "tileset" | "material" => "images",
                    "audio" | "voice" => "audio",
                    "video" => "video",
                    "code" => "code",
                    _ => "other",
                };

                let extension = Path::new(file_path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("bin");
                let dest_file_name = format!("{}_{}.{}", sanitize_filename(&asset.name), &asset.id.into_uuid().to_string()[..8], extension);
                let dest_path = export_dir.join(subdir).join(&dest_file_name);

                // Copy file
                if let Err(e) = std::fs::copy(source, &dest_path) {
                    tracing::warn!("Failed to copy asset {}: {}", asset.name, e);
                    continue;
                }

                manifest_assets.push(AssetManifestEntry {
                    name: asset.name.clone(),
                    kind: asset.kind.as_str().to_string(),
                    file_path: format!("{}/{}", subdir, dest_file_name),
                    width: asset.width,
                    height: asset.height,
                    file_size: asset.file_size,
                    metadata: asset.metadata.clone(),
                });
            }
        }
    }

    // Create manifest.json
    let manifest = ExportManifest {
        version: "1.0".to_string(),
        project_name: project.name.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        asset_count: manifest_assets.len(),
        assets: manifest_assets,
    };

    let manifest_json = serde_json::to_string_pretty(&manifest)
        .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
    std::fs::write(export_dir.join("manifest.json"), manifest_json)
        .map_err(|e| format!("Failed to write manifest: {}", e))?;

    // Generate HTML gallery if requested
    if request.include_html_gallery {
        let gallery_html = generate_html_gallery(&project.name.to_string(), &manifest);
        std::fs::write(export_dir.join("index.html"), gallery_html)
            .map_err(|e| format!("Failed to write gallery: {}", e))?;
    }

    // Create ZIP archive
    let zip_path = Path::new(&output_path);
    let zip_file = std::fs::File::create(zip_path)
        .map_err(|e| format!("Failed to create ZIP file: {}", e))?;
    let mut zip = ZipWriter::new(zip_file);
    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Walk the export directory and add files to ZIP
    add_directory_to_zip(&mut zip, &export_dir, "", &options)?;

    // Finish ZIP
    zip.finish().map_err(|e| format!("Failed to finish ZIP: {}", e))?;

    // Get file size
    let file_size = std::fs::metadata(zip_path)
        .map_err(|e| format!("Failed to get ZIP file size: {}", e))?
        .len();

    // Clean up temp directory
    if let Err(e) = std::fs::remove_dir_all(&temp_dir) {
        tracing::warn!("Failed to clean up temp dir: {}", e);
    }

    Ok(ExportProjectResponse {
        output_path,
        file_size_bytes: file_size,
        asset_count: manifest.asset_count,
        manifest_path: "manifest.json".to_string(),
    })
}

/// Opens itch.io game creation page in the default browser.
#[tauri::command]
pub async fn open_itch_io(app: tauri::AppHandle) -> Result<(), String> {
    tauri_plugin_shell::ShellExt::shell(&app)
        .open("https://itch.io/game/new", None)
        .map_err(|e| format!("Failed to open itch.io: {}", e))
}

// Helper functions

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn add_directory_to_zip(
    zip: &mut ZipWriter<std::fs::File>,
    dir: &Path,
    prefix: &str,
    options: &SimpleFileOptions,
) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("Failed to read directory: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let path = entry.path();
        let name = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file name")?;

        let archive_name = if prefix.is_empty() {
            name.to_string()
        } else {
            format!("{}/{}", prefix, name)
        };

        if path.is_dir() {
            // Add directory entry
            zip.add_directory(&archive_name, *options)
                .map_err(|e| format!("Failed to add directory to ZIP: {}", e))?;
            add_directory_to_zip(zip, &path, &archive_name, options)?;
        } else {
            // Add file entry
            let data = std::fs::read(&path)
                .map_err(|e| format!("Failed to read file {}: {}", name, e))?;
            zip.start_file(&archive_name, *options)
                .map_err(|e| format!("Failed to start file in ZIP: {}", e))?;
            zip.write(&data)
                .map_err(|e| format!("Failed to write file to ZIP: {}", e))?;
        }
    }

    Ok(())
}

fn generate_html_gallery(project_name: &str, manifest: &ExportManifest) -> String {
    let mut html = String::new();

    html.push_str(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>"#);
    html.push_str(project_name);
    html.push_str(r#" - Asset Gallery</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #1a1a2e;
            color: #e0e0e0;
            min-height: 100vh;
            padding: 2rem;
        }
        h1 {
            text-align: center;
            margin-bottom: 2rem;
            color: #fff;
        }
        .gallery {
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
            gap: 1.5rem;
            max-width: 1400px;
            margin: 0 auto;
        }
        .card {
            background: #16213e;
            border-radius: 12px;
            overflow: hidden;
            box-shadow: 0 4px 6px rgba(0,0,0,0.3);
            transition: transform 0.2s;
        }
        .card:hover { transform: translateY(-4px); }
        .card img, .card audio, .card video, .card pre {
            width: 100%;
            display: block;
        }
        .card img { height: 150px; object-fit: cover; }
        .card audio { height: 40px; }
        .card video { height: 150px; object-fit: cover; }
        .card pre {
            padding: 1rem;
            font-size: 0.7rem;
            overflow: hidden;
            max-height: 150px;
            background: #0f0f23;
            white-space: pre-wrap;
            word-break: break-all;
        }
        .card-info {
            padding: 1rem;
        }
        .card-name {
            font-weight: 600;
            margin-bottom: 0.25rem;
            color: #fff;
        }
        .card-meta {
            font-size: 0.8rem;
            color: #888;
        }
        .kind-badge {
            display: inline-block;
            padding: 0.2rem 0.5rem;
            background: #e94560;
            border-radius: 4px;
            font-size: 0.7rem;
            margin-top: 0.5rem;
        }
        .stats {
            text-align: center;
            margin-bottom: 2rem;
            color: #888;
        }
    </style>
</head>
<body>
    <h1>"#);
    html.push_str(project_name);
    html.push_str(r#" - Asset Gallery</h1>
    <p class="stats">Total assets: "#);
    html.push_str(&manifest.asset_count.to_string());
    html.push_str(r#"</p>
    <div class="gallery">
"#);

    for asset in &manifest.assets {
        html.push_str(r#"        <div class="card">
"#);

        match asset.kind.as_str() {
            "image" | "sprite" | "tileset" | "material" => {
                html.push_str(&format!(r#"            <img src="{}" alt="{}" onerror="this.style.display='none'">
"#, asset.file_path, asset.name));
            }
            "audio" | "voice" => {
                html.push_str(&format!(r#"            <audio controls src="{}">
                Your browser does not support audio.
            </audio>
"#, asset.file_path));
            }
            "video" => {
                html.push_str(&format!(r#"            <video controls src="{}">
                Your browser does not support video.
            </video>
"#, asset.file_path));
            }
            "code" => {
                // Try to read code content for preview
                html.push_str(&format!(r#"            <pre>{}</pre>
"#, asset.name));
            }
            _ => {
                html.push_str(&format!(r#"            <div style="height:150px;display:flex;align-items:center;justify-content:center;background:#0f0f23;color:#888;">No preview</div>
"#));
            }
        }

        html.push_str(&format!(r#"            <div class="card-info">
                <div class="card-name">{}</div>
"#, asset.name));

        if let (Some(w), Some(h)) = (asset.width, asset.height) {
            html.push_str(&format!(r#"                <div class="card-meta">{} x {}</div>
"#, w, h));
        }

        if let Some(size) = asset.file_size {
            html.push_str(&format!(r#"                <div class="card-meta">{} bytes</div>
"#, size));
        }

        html.push_str(&format!(r#"                <span class="kind-badge">{}</span>
            </div>
        </div>
"#, asset.kind));
    }

    html.push_str(r#"    </div>
</body>
</html>"#);

    html
}
