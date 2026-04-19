//! Data transfer objects for IPC communication.
//!
//! This module re-exports all public DTO types from submodules.

pub mod projects;
pub mod jobs;
pub mod assets;
pub mod image_gen;
pub mod video;
pub mod audio;
pub mod image_processing;
pub mod tiles;
pub mod sprite_sheet;
pub mod animations;
pub mod atlas;
pub mod quick_sprites;
pub mod code;
pub mod materials;
pub mod publish;
pub mod renderer_3d;

// Re-export all types for backwards compatibility
pub use projects::{CreateProjectRequest, ProjectResponse};
pub use jobs::{CreateJobRequest, JobResponse};
pub use assets::{
    AddToCollectionRequest, AssetLineageResponse, AssetResponse, CollectionCreateRequest,
    CollectionResponse, ImportAssetRequest, RegisterAssetRequest, TagAssetRequest,
    UntagAssetRequest,
};
pub use image_gen::GenerateImageRequest;
pub use video::GenerateVideoRequest;
pub use audio::{GenerateAudioRequest, GenerateTtsRequest, AudioGenParamsDto, TtsParamsDto};
pub use image_processing::{
    RemoveBackgroundRequest, PaletteMode, DitheringMode, ConvertPixelArtRequest,
    InpaintRequest, OutpaintDirection, OutpaintRequest,
};
pub use tiles::{GenerateTileRequest, SeamlessMode, SeamlessTextureRequest};
pub use sprite_sheet::{
    OutputFormat, GenerateSpriteSheetRequest, SortOrder, SliceMode,
    GridSliceParams, AutoDetectSliceParams, SliceSpriteSheetRequest,
};
pub use animations::{
    CreateAnimationRequest, UpdateAnimationRequest, ExportAnimationRequest, ExportAnimationFormat,
};
pub use atlas::{AtlasSortMode, PackAtlasOptions, PackAtlasRequest};
pub use quick_sprites::{
    QuickSpritesMode, QuickSpritesOutputFormat, QuickSpritesOptions, QuickSpritesRequest,
};
pub use code::{CodeAgentRequest, GenerateCodeRequest};
pub use materials::GenerateMaterialRequest;
pub use publish::{ExportProjectResponse, ExportProjectRequest};
pub use renderer_3d::{CameraAngle, Render3dRequest};
