//! 3D to 2D Renderer Worker.
//!
//! Renders GLTF/GLB/OBJ models from configurable angles to sprite atlases.
//! Uses raw wgpu for headless offscreen rendering.

use std::path::{Path, PathBuf};

use artifex_job_queue::Job;
use artifex_shared_kernel::AppError;
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use gltf;
use image::{DynamicImage, RgbaImage};
use serde::{Deserialize, Serialize};
use texture_packer::exporter::ImageExporter;
use texture_packer::{TexturePacker, TexturePackerConfig};
use tobj::LoadOptions;
use wgpu::util::DeviceExt;
use wgpu::{DeviceDescriptor, Extent3d, ImageCopyBuffer, ImageCopyTexture, ImageDataLayout, InstanceDescriptor, Origin3d, TextureAspect, TextureUsages};

use crate::dto::CameraAngle;

use super::traits::{JobFuture, JobResult, JobWorker, WorkerCategory};

// ============================================================================
// Constants
// ============================================================================

const VERTEX_SHADER: &str = r#"
struct Uniforms {
    view_proj: mat4x4<f32>,
};
@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    out.color = input.color;
    return out;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(input.color, 1.0);
}
"#;

// ============================================================================
// Types
// ============================================================================

/// Operation payload for render_3d jobs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Render3dOperation {
    pub project_id: String,
    pub model_file_path: String,
    pub camera_preset: String,
    #[serde(default)]
    pub custom_angles: Option<Vec<CameraAngle>>,
    #[serde(default = "default_output_width")]
    pub output_width: u32,
    #[serde(default = "default_output_height")]
    pub output_height: u32,
    #[serde(default)]
    pub animation_name: Option<String>,
    #[serde(default = "default_animation_fps")]
    pub animation_fps: u32,
}

fn default_output_width() -> u32 {
    256
}

fn default_output_height() -> u32 {
    256
}

fn default_animation_fps() -> u32 {
    12
}

/// Vertex format for rendering.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

/// Uniforms passed to the shader.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Uniforms {
    view_proj: [[f32; 4]; 4],
}

/// Mesh data extracted from a model.
struct MeshData {
    vertices: Vec<Vertex>,
    indices: Vec<u32>,
}

/// Camera parameters for rendering.
struct Camera {
    view: Mat4,
    projection: Mat4,
}

/// Region within an atlas manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AtlasRegion {
    asset_id: String,
    name: String,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
    source_width: u32,
    source_height: u32,
    rotated: bool,
}

/// Atlas manifest JSON structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AtlasManifest {
    version: u32,
    atlas_name: String,
    atlas_width: u32,
    atlas_height: u32,
    regions: Vec<AtlasRegion>,
}

// ============================================================================
// Renderer 3D Worker
// ============================================================================

pub struct Renderer3dWorker {
    assets_dir: String,
}

impl Renderer3dWorker {
    pub fn new(assets_dir: String) -> Self {
        Self { assets_dir }
    }
}

impl JobWorker for Renderer3dWorker {
    fn can_handle(&self, job_type: &str) -> bool {
        job_type == "render_3d"
    }

    fn category(&self) -> WorkerCategory {
        WorkerCategory::CpuIntensive
    }

    fn process(&self, job: &Job) -> JobFuture {
        let assets_dir = self.assets_dir.clone();
        let job_id = job.id;
        let operation = job.operation.clone();

        Box::pin(async move {
            let op: Render3dOperation = serde_json::from_value(operation)
                .map_err(|e| AppError::validation(format!("Invalid render_3d operation: {}", e)))?;

            tracing::info!(
                "Renderer3dWorker processing job {} for project {}",
                job_id.into_uuid(),
                op.project_id
            );

            let worker = Renderer3dWorker::new(assets_dir);
            worker.process_render_job(op).await
        })
    }
}

impl Renderer3dWorker {
    async fn process_render_job(&self, op: Render3dOperation) -> Result<JobResult, AppError> {
        // Validate model file exists
        let model_path = PathBuf::from(&op.model_file_path);
        if !model_path.exists() {
            return Err(AppError::validation(format!(
                "Model file not found: {}",
                op.model_file_path
            )));
        }

        // Determine model format
        let extension = model_path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_lowercase())
            .unwrap_or_default();

        let meshes = match extension.as_str() {
            "gltf" | "glb" => self.load_gltf(&model_path)?,
            "obj" => self.load_obj(&model_path)?,
            _ => {
                return Err(AppError::validation(format!(
                    "Unsupported model format: {} (supported: gltf, glb, obj)",
                    extension
                )));
            }
        };

        // Generate camera angles based on preset
        let camera_angles = match op.camera_preset.as_str() {
            "isometric" => generate_isometric_angles(),
            "topdown" => generate_topdown_angles(),
            "custom" => {
                op.custom_angles
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| (a.yaw_degrees, a.pitch_degrees))
                    .collect()
            }
            _ => {
                return Err(AppError::validation(format!(
                    "Invalid camera preset: {} (valid: isometric, topdown, custom)",
                    op.camera_preset
                )));
            }
        };

        // Create output directory
        let output_dir = PathBuf::from(&self.assets_dir)
            .join(&op.project_id)
            .join("sprites");
        tokio::fs::create_dir_all(&output_dir).await.map_err(|e| {
            AppError::io_error(format!("Failed to create output directory: {}", e))
        })?;

        // Initialize wgpu headless
        let instance = wgpu::Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: None,
                ..Default::default()
            })
            .await
            .ok_or_else(|| {
                AppError::internal(
                    "No GPU adapter found. Ensure Vulkan drivers are installed.".to_string(),
                )
            })?;

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor::default(), None)
            .await
            .map_err(|e| AppError::internal(format!("GPU device error: {}", e)))?;

        // Calculate bounding box for camera fitting
        let bbox = calculate_bounding_box(&meshes);
        let center = (bbox.0 + bbox.1) * 0.5;
        let size = bbox.1 - bbox.0;
        let max_dim = size.x.max(size.y).max(size.z);
        let scale = if max_dim > 0.0 { 2.0 / max_dim } else { 1.0 };

        // Collect all frame images
        let mut frame_images: Vec<DynamicImage> = Vec::new();

        // Static rendering - render from each camera angle
        for (yaw, pitch) in &camera_angles {
            let rgba = self.render_frame(
                &device,
                &queue,
                &meshes,
                center,
                scale,
                *yaw,
                *pitch,
                op.output_width,
                op.output_height,
            )?;
            frame_images.push(DynamicImage::ImageRgba8(rgba));
        }

        if frame_images.is_empty() {
            return Err(AppError::validation("No frames to render".to_string()));
        }

        // Pack frames into a sprite atlas
        let atlas_name = format!(
            "render_{}_{}",
            model_path.file_stem().unwrap_or_default().to_string_lossy(),
            uuid::Uuid::new_v4().to_string()[..8].to_string()
        );

        let config = TexturePackerConfig {
            max_width: 4096,
            max_height: 4096,
            allow_rotation: false,
            border_padding: 1,
            ..Default::default()
        };

        let mut packer = TexturePacker::new_skyline(config);

        for (idx, img) in frame_images.iter().enumerate() {
            let name = format!("frame_{:04}", idx);
            packer
                .pack_own(name, img.clone())
                .map_err(|e| AppError::internal(format!("Failed to pack frame {}: {:?}", idx, e)))?;
        }

        // Export atlas
        let atlas = ImageExporter::export(&packer, None)
            .map_err(|e| AppError::internal(format!("Failed to export atlas: {:?}", e)))?;

        let atlas_width = atlas.width();
        let atlas_height = atlas.height();

        // Save atlas PNG
        let atlas_filename = format!("{}.png", atlas_name);
        let atlas_path = output_dir.join(&atlas_filename);
        atlas
            .save(&atlas_path)
            .map_err(|e| AppError::internal(format!("Failed to save atlas: {}", e)))?;

        // Build manifest
        let frames_map = packer.get_frames();
        let mut regions: Vec<AtlasRegion> = Vec::new();

        for (idx, img) in frame_images.iter().enumerate() {
            let name = format!("frame_{:04}", idx);
            if let Some(frame) = frames_map.get(&name) {
                regions.push(AtlasRegion {
                    asset_id: format!("frame_{:04}", idx),
                    name: name.clone(),
                    x: frame.frame.x,
                    y: frame.frame.y,
                    w: frame.frame.w,
                    h: frame.frame.h,
                    source_width: img.width(),
                    source_height: img.height(),
                    rotated: false,
                });
            }
        }

        let region_count = regions.len();

        // Write manifest JSON
        let manifest = AtlasManifest {
            version: 1,
            atlas_name: atlas_name.clone(),
            atlas_width,
            atlas_height,
            regions,
        };

        let manifest_filename = format!("{}.json", atlas_name);
        let manifest_path = output_dir.join(&manifest_filename);
        let manifest_json =
            serde_json::to_string_pretty(&manifest).map_err(|e| AppError::internal(format!(
                "Failed to serialize manifest: {}",
                e
            )))?;
        std::fs::write(&manifest_path, manifest_json)
            .map_err(|e| AppError::io_error(format!("Failed to write manifest: {}", e)))?;

        let output_files = vec![atlas_path, manifest_path];

        let metadata = serde_json::json!({
            "operation": "render_3d",
            "atlas_name": atlas_name,
            "project_id": op.project_id,
            "atlas_width": atlas_width,
            "atlas_height": atlas_height,
            "frame_count": region_count,
            "camera_preset": op.camera_preset,
            "model_file_path": op.model_file_path,
            "output_width": op.output_width,
            "output_height": op.output_height,
            "atlas_manifest": manifest,
        });

        Ok(JobResult::with_metadata(output_files, metadata))
    }
}

impl Renderer3dWorker {
    /// Renders a single frame from the given camera angle.
    fn render_frame(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        meshes: &[MeshData],
        center: Vec3,
        scale: f32,
        yaw_degrees: f32,
        pitch_degrees: f32,
        width: u32,
        height: u32,
    ) -> Result<RgbaImage, AppError> {
        // Compute camera matrices
        let camera = compute_camera(yaw_degrees, pitch_degrees, center, scale);

        // Create offscreen texture
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("output_texture"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        // Create output buffer for readback
        let buffer_size = 4 * width as u64 * height as u64;
        let output_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("output_buffer"),
            size: buffer_size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });

        // Compile shader module
        let shader_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("render_3d_shader"),
            source: wgpu::ShaderSource::Wgsl(VERTEX_SHADER.into()),
        });

        // Create bind group layout and bind group
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("uniforms_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("render_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("render_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader_module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<Vertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: std::mem::size_of::<[f32; 3]>() as u64,
                            shader_location: 1,
                        },
                    ],
                }],
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader_module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::all(),
                })],
            }),
            multiview: None,
            cache: None,
        });

        // For each mesh, create vertex buffer, bind group, and render
        for mesh in meshes {
            // Create vertex buffer
            let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("vertex_buffer"),
                contents: bytemuck::cast_slice(&mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });

            // Create index buffer
            let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("index_buffer"),
                contents: bytemuck::cast_slice(&mesh.indices),
                usage: wgpu::BufferUsages::INDEX,
            });

            // Create uniform buffer
            let view_proj = camera.projection * camera.view;
            let uniforms = Uniforms {
                view_proj: [
                    [view_proj.col(0).x, view_proj.col(0).y, view_proj.col(0).z, view_proj.col(0).w],
                    [view_proj.col(1).x, view_proj.col(1).y, view_proj.col(1).z, view_proj.col(1).w],
                    [view_proj.col(2).x, view_proj.col(2).y, view_proj.col(2).z, view_proj.col(2).w],
                    [view_proj.col(3).x, view_proj.col(3).y, view_proj.col(3).z, view_proj.col(3).w],
                ],
            };

            let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("uniform_buffer"),
                contents: bytemuck::cast_slice(&[uniforms]),
                usage: wgpu::BufferUsages::UNIFORM,
            });

            let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("uniforms_bind_group"),
                layout: &bind_group_layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform_buffer.as_entire_binding(),
                }],
            });

            // Create command encoder
            let mut encoder = device.create_command_encoder(&Default::default());

            // Render pass
            {
                let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("render_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &texture.create_view(&Default::default()),
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 0.0, // Transparent background
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

                render_pass.set_pipeline(&render_pipeline);
                render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
                render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.set_bind_group(0, &bind_group, &[]);
                render_pass.draw_indexed(0..mesh.indices.len() as u32, 0, 0..1);
            }

            // Copy texture to buffer for readback
            encoder.copy_texture_to_buffer(
                ImageCopyTexture {
                    texture: &texture,
                    mip_level: 0,
                    origin: Origin3d::ZERO,
                    aspect: TextureAspect::All,
                },
                ImageCopyBuffer {
                    buffer: &output_buffer,
                    layout: ImageDataLayout {
                        offset: 0,
                        bytes_per_row: Some(4 * width),
                        rows_per_image: Some(height),
                    },
                },
                Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
            );

            queue.submit(Some(encoder.finish()));
        }

        // Read back the buffer
        let buffer_slice = output_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            let _ = tx.send(result);
        });
        device.poll(wgpu::Maintain::Wait);
        rx.recv().unwrap().map_err(|e| AppError::internal(format!("Buffer map error: {}", e)))?;

        let data = buffer_slice.get_mapped_range();
        let rgba_image = RgbaImage::from_raw(width, height, data.to_vec())
            .ok_or_else(|| AppError::internal("Failed to create RGBA image from buffer".to_string()))?;

        Ok(rgba_image)
    }

    /// Loads a GLTF or GLB model.
    fn load_gltf(&self, path: &Path) -> Result<Vec<MeshData>, AppError> {
        let (gltf, buffers, _animations) = gltf::import(path)
            .map_err(|e| AppError::internal(format!("Failed to load GLTF {}: {}", path.display(), e)))?;

        let mut meshes = Vec::new();

        for mesh in gltf.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // Get position attribute
                let positions = reader
                    .read_positions()
                    .ok_or_else(|| AppError::internal("Mesh has no positions".to_string()))?;

                // Get color attribute (fallback to white)
                let colors: Vec<[f32; 3]> = reader
                    .read_colors(0)
                    .map(|c| {
                        c.into_rgba_f32()
                            .map(|[r, g, b, _]| [r, g, b])
                            .collect()
                    })
                    .unwrap_or_else(|| {
                        positions.clone().map(|_| [1.0, 1.0, 1.0]).collect()
                    });

                let vertices: Vec<Vertex> = positions
                    .zip(colors.iter().copied())
                    .map(|(pos, color)| Vertex {
                        position: pos,
                        color,
                    })
                    .collect();

                let indices: Vec<u32> = reader
                    .read_indices()
                    .map(|i| i.into_u32().collect())
                    .unwrap_or_else(|| (0..vertices.len() as u32).collect());

                meshes.push(MeshData { vertices, indices });
            }
        }

        Ok(meshes)
    }

    /// Loads an OBJ model.
    fn load_obj(&self, path: &Path) -> Result<Vec<MeshData>, AppError> {
        let (models, _) = tobj::load_obj(path, &LoadOptions::default())
            .map_err(|e| AppError::internal(format!("Failed to load OBJ {}: {}", path.display(), e)))?;

        let mut meshes = Vec::new();

        for model in &models {
            let mesh = &model.mesh;

            let positions: Vec<[f32; 3]> = mesh
                .positions
                .chunks(3)
                .map(|c| [c[0], c[1], c[2]])
                .collect();

            // Use vertex color if available, otherwise white
            let colors: Vec<[f32; 3]> = if !mesh.vertex_color.is_empty() {
                mesh.vertex_color
                    .chunks(3)
                    .map(|c| [c[0], c[1], c[2]])
                    .collect()
            } else {
                positions.iter().map(|_| [1.0, 1.0, 1.0]).collect()
            };

            let vertices: Vec<Vertex> = positions
                .into_iter()
                .zip(colors.into_iter())
                .map(|(pos, color)| Vertex {
                    position: pos,
                    color,
                })
                .collect();

            let indices: Vec<u32> = mesh
                .indices
                .iter()
                .map(|&i| i as u32)
                .collect();

            meshes.push(MeshData { vertices, indices });
        }

        Ok(meshes)
    }
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Generates camera angles for isometric preset (8 directions).
fn generate_isometric_angles() -> Vec<(f32, f32)> {
    let pitch = 35.264_f32.atan2(1.0).to_degrees(); // ~35.264 degrees
    let yaw_steps = [0.0, 45.0, 90.0, 135.0, 180.0, 225.0, 270.0, 315.0];
    yaw_steps
        .iter()
        .copied()
        .map(|yaw| (yaw, pitch))
        .collect()
}

/// Generates camera angles for top-down preset (4 directions).
fn generate_topdown_angles() -> Vec<(f32, f32)> {
    let pitch = 90.0; // Looking straight down
    let yaw_steps = [0.0, 90.0, 180.0, 270.0];
    yaw_steps
        .iter()
        .copied()
        .map(|yaw| (yaw, pitch))
        .collect()
}

/// Computes camera view and projection matrices.
fn compute_camera(yaw_degrees: f32, pitch_degrees: f32, center: Vec3, scale: f32) -> Camera {
    let yaw_rad = yaw_degrees.to_radians();
    let pitch_rad = pitch_degrees.to_radians();

    // Camera position on a sphere around the model
    let distance = 5.0;
    let x = distance * yaw_rad.cos() * pitch_rad.cos();
    let y = distance * pitch_rad.sin();
    let z = distance * yaw_rad.sin() * pitch_rad.cos();

    let eye = center + Vec3::new(x, y, z) / scale;
    let target = center;

    // Look-at view matrix
    let view = Mat4::look_at_rh(eye, target, Vec3::Y);

    // Perspective projection
    let aspect = 1.0; // Square output
    let fov = 45.0_f32.to_radians();
    let projection = Mat4::perspective_rh(fov, aspect, 0.1, 100.0);

    Camera { view, projection }
}

/// Calculates bounding box of all meshes.
fn calculate_bounding_box(meshes: &[MeshData]) -> (Vec3, Vec3) {
    let mut min_pos = Vec3::splat(f32::MAX);
    let mut max_pos = Vec3::splat(f32::MIN);

    for mesh in meshes {
        for vertex in &mesh.vertices {
            let pos = Vec3::from(vertex.position);
            min_pos = min_pos.min(pos);
            max_pos = max_pos.max(pos);
        }
    }

    if min_pos == Vec3::splat(f32::MAX) {
        return (Vec3::ZERO, Vec3::ONE);
    }

    (min_pos, max_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_can_handle() {
        let worker = Renderer3dWorker::new("/tmp/assets".to_string());
        assert!(worker.can_handle("render_3d"));
        assert!(!worker.can_handle("pack_atlas"));
        assert!(!worker.can_handle("sprite_generate"));
    }

    #[test]
    fn test_generate_isometric_angles() {
        let angles = generate_isometric_angles();
        assert_eq!(angles.len(), 8);
        // All should have same pitch
        let pitch = angles[0].1;
        for (_, p) in &angles {
            assert_eq!(*p, pitch);
        }
    }

    #[test]
    fn test_generate_topdown_angles() {
        let angles = generate_topdown_angles();
        assert_eq!(angles.len(), 4);
        // All should have 90 degree pitch
        for (_, p) in &angles {
            assert_eq!(*p, 90.0);
        }
    }

    #[test]
    fn test_camera_computation() {
        let camera = compute_camera(45.0, 35.264, Vec3::ZERO, 1.0);
        // Just verify it doesn't panic - Mat4 always exists
        assert!(camera.view != Mat4::ZERO);
        assert!(camera.projection != Mat4::ZERO);
    }
}
