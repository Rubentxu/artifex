//! Asset model.

use serde::{Deserialize, Serialize};

use artifex_shared_kernel::{AssetId, ProjectId, Timestamp};

/// Asset kind enumeration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AssetKind {
    /// A 2D image.
    Image,
    /// A sprite sheet or sprite animation.
    Sprite,
    /// A tile set for 2D maps.
    Tileset,
    /// A material definition.
    Material,
    /// An audio resource.
    Audio,
    /// Voice acting audio.
    Voice,
    /// A video resource.
    Video,
    /// A code/script file.
    Code,
    /// An animation clip assembled from ordered frame assets.
    Animation,
    /// Some other type of asset.
    Other,
}

impl AssetKind {
    /// Converts AssetKind to a string for database storage.
    pub fn as_str(&self) -> &'static str {
        match self {
            AssetKind::Image => "image",
            AssetKind::Sprite => "sprite",
            AssetKind::Tileset => "tileset",
            AssetKind::Material => "material",
            AssetKind::Audio => "audio",
            AssetKind::Voice => "voice",
            AssetKind::Video => "video",
            AssetKind::Code => "code",
            AssetKind::Animation => "animation",
            AssetKind::Other => "other",
        }
    }

    /// Parses a string to AssetKind.
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "image" => Some(AssetKind::Image),
            "sprite" => Some(AssetKind::Sprite),
            "tileset" => Some(AssetKind::Tileset),
            "material" => Some(AssetKind::Material),
            "audio" => Some(AssetKind::Audio),
            "voice" => Some(AssetKind::Voice),
            "video" => Some(AssetKind::Video),
            "code" => Some(AssetKind::Code),
            "animation" => Some(AssetKind::Animation),
            "other" => Some(AssetKind::Other),
            _ => None,
        }
    }
}

/// Metadata for Animation assets.
///
/// This struct is serialized to JSON and stored in the `Asset.metadata` field
/// when the asset kind is `AssetKind::Animation`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnimationMetadata {
    /// Animation display name.
    pub name: String,
    /// Ordered list of asset IDs that constitute the animation frames.
    pub frame_asset_ids: Vec<String>,
    /// Duration of each frame in milliseconds.
    /// Must have the same length as `frame_asset_ids`.
    pub frame_durations_ms: Vec<u32>,
    /// Whether the animation should loop.
    #[serde(default)]
    pub loop_animation: bool,
    /// Total duration of the animation in milliseconds (computed).
    pub total_duration_ms: u32,
    /// Default FPS used when this animation was created from a fixed-FPS source.
    /// None if durations were set manually.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_fps: Option<u16>,
}

impl AnimationMetadata {
    /// Creates a new AnimationMetadata with computed total_duration_ms.
    ///
    /// # Errors
    /// Returns an error if `frame_asset_ids` is empty or if the lengths of
    /// `frame_asset_ids` and `frame_durations_ms` don't match.
    pub fn new(
        name: String,
        frame_asset_ids: Vec<String>,
        frame_durations_ms: Vec<u32>,
        loop_animation: bool,
        default_fps: Option<u16>,
    ) -> Result<Self, &'static str> {
        if frame_asset_ids.is_empty() {
            return Err("Animation must have at least one frame");
        }
        if frame_asset_ids.len() != frame_durations_ms.len() {
            return Err("frame_asset_ids and frame_durations_ms must have the same length");
        }
        if frame_durations_ms.iter().any(|&d| d == 0) {
            return Err("All frame durations must be positive");
        }

        let total_duration_ms = frame_durations_ms.iter().sum();

        Ok(Self {
            name,
            frame_asset_ids,
            frame_durations_ms,
            loop_animation,
            total_duration_ms,
            default_fps,
        })
    }

    /// Creates AnimationMetadata with uniform frame durations computed from FPS.
    pub fn with_uniform_fps(
        name: String,
        frame_asset_ids: Vec<String>,
        fps: u16,
        loop_animation: bool,
    ) -> Result<Self, &'static str> {
        if frame_asset_ids.is_empty() {
            return Err("Animation must have at least one frame");
        }

        let frame_duration_ms = if fps > 0 { 1000 / fps as u32 } else { 100 };
        let frame_durations_ms = vec![frame_duration_ms; frame_asset_ids.len()];
        let total_duration_ms = frame_duration_ms * frame_asset_ids.len() as u32;

        Ok(Self {
            name,
            frame_asset_ids,
            frame_durations_ms,
            loop_animation,
            total_duration_ms,
            default_fps: Some(fps),
        })
    }
}

/// A validated asset model with file metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique asset identifier.
    pub id: AssetId,
    /// The project this asset belongs to.
    pub project_id: ProjectId,
    /// Asset display name.
    pub name: String,
    /// Kind/type of asset.
    pub kind: AssetKind,
    /// Path to the actual file (relative to project root or absolute).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    /// Arbitrary metadata stored as JSON.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// File size in bytes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_size: Option<u64>,
    /// Image width in pixels (for image assets).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// Image height in pixels (for image assets).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// When the asset was created.
    pub created_at: Timestamp,
    /// Tags for organization.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Source of import (e.g., "uploaded", "generated").
    #[serde(default)]
    pub import_source: String,
    /// Collection this asset belongs to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_id: Option<String>,
    /// Parent asset ID if this asset was derived from another.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub derived_from_asset_id: Option<String>,
}

impl Asset {
    /// Creates a new asset with minimal fields.
    ///
    /// Use this for assets where the file hasn't been saved yet or no metadata is available.
    pub fn new(project_id: ProjectId, name: impl Into<String>, kind: AssetKind) -> Self {
        Self {
            id: AssetId::new(),
            project_id,
            name: name.into(),
            kind,
            file_path: None,
            metadata: None,
            file_size: None,
            width: None,
            height: None,
            created_at: Timestamp::now(),
            tags: Vec::new(),
            import_source: "uploaded".to_string(),
            collection_id: None,
            derived_from_asset_id: None,
        }
    }

    /// Registers a new asset with required fields validated.
    ///
    /// # Errors
    /// Returns an error if the name is empty.
    pub fn register(
        project_id: ProjectId,
        name: impl Into<String>,
        kind: AssetKind,
    ) -> Result<Self, &'static str> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err("Asset name cannot be empty");
        }
        Ok(Self::new(project_id, name, kind))
    }

    /// Creates a new image asset with image-specific metadata.
    pub fn with_image_metadata(
        project_id: ProjectId,
        name: impl Into<String>,
        file_path: impl Into<String>,
        width: u32,
        height: u32,
        file_size: u64,
    ) -> Self {
        Self {
            id: AssetId::new(),
            project_id,
            name: name.into(),
            kind: AssetKind::Image,
            file_path: Some(file_path.into()),
            metadata: Some(serde_json::json!({
                "width": width,
                "height": height,
            })),
            file_size: Some(file_size),
            width: Some(width),
            height: Some(height),
            created_at: Timestamp::now(),
            tags: Vec::new(),
            import_source: "uploaded".to_string(),
            collection_id: None,
            derived_from_asset_id: None,
        }
    }

    /// Sets the file path for the asset.
    pub fn with_file_path(mut self, path: impl Into<String>) -> Self {
        self.file_path = Some(path.into());
        self
    }

    /// Sets the metadata for the asset.
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets the file size for the asset.
    pub fn with_file_size(mut self, size: u64) -> Self {
        self.file_size = Some(size);
        self
    }

    /// Sets image dimensions for the asset.
    pub fn with_dimensions(mut self, width: u32, height: u32) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Sets the tags for the asset.
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Sets the import source for the asset.
    pub fn with_import_source(mut self, source: String) -> Self {
        self.import_source = source;
        self
    }

    /// Sets the collection ID for the asset.
    pub fn with_collection_id(mut self, collection_id: Option<String>) -> Self {
        self.collection_id = collection_id;
        self
    }

    /// Sets the derived_from_asset_id for the asset.
    pub fn with_derived_from(mut self, derived_from: Option<String>) -> Self {
        self.derived_from_asset_id = derived_from;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_creation() {
        let project_id = ProjectId::new();
        let asset = Asset::new(project_id, "player.png", AssetKind::Image);

        assert_eq!(asset.name, "player.png");
        assert_eq!(asset.kind, AssetKind::Image);
        assert_eq!(asset.project_id, project_id);
        assert!(asset.file_path.is_none());
        assert!(asset.metadata.is_none());
        assert!(asset.file_size.is_none());
    }

    #[test]
    fn test_asset_kind_serialization() {
        assert_eq!(AssetKind::Image.as_str(), "image");
        assert_eq!(AssetKind::Sprite.as_str(), "sprite");
        assert_eq!(AssetKind::Tileset.as_str(), "tileset");
        assert_eq!(AssetKind::Material.as_str(), "material");
        assert_eq!(AssetKind::Audio.as_str(), "audio");
        assert_eq!(AssetKind::Voice.as_str(), "voice");
        assert_eq!(AssetKind::Video.as_str(), "video");
        assert_eq!(AssetKind::Code.as_str(), "code");
        assert_eq!(AssetKind::Animation.as_str(), "animation");
        assert_eq!(AssetKind::Other.as_str(), "other");
    }

    #[test]
    fn test_asset_kind_from_str() {
        assert_eq!(AssetKind::from_str("image"), Some(AssetKind::Image));
        assert_eq!(AssetKind::from_str("sprite"), Some(AssetKind::Sprite));
        assert_eq!(AssetKind::from_str("tileset"), Some(AssetKind::Tileset));
        assert_eq!(AssetKind::from_str("material"), Some(AssetKind::Material));
        assert_eq!(AssetKind::from_str("audio"), Some(AssetKind::Audio));
        assert_eq!(AssetKind::from_str("voice"), Some(AssetKind::Voice));
        assert_eq!(AssetKind::from_str("video"), Some(AssetKind::Video));
        assert_eq!(AssetKind::from_str("code"), Some(AssetKind::Code));
        assert_eq!(AssetKind::from_str("animation"), Some(AssetKind::Animation));
        assert_eq!(AssetKind::from_str("other"), Some(AssetKind::Other));
        assert_eq!(AssetKind::from_str("unknown"), None);
    }

    #[test]
    fn test_asset_kind_animation_round_trip() {
        // SC-1a: Animation kind round-trips
        let kind = AssetKind::Animation;
        let serialized = kind.as_str();
        assert_eq!(serialized, "animation");
        let deserialized = AssetKind::from_str(serialized);
        assert_eq!(deserialized, Some(AssetKind::Animation));
    }

    #[test]
    fn test_asset_register_valid() {
        let project_id = ProjectId::new();
        let asset = Asset::register(project_id, "test.png", AssetKind::Image).unwrap();
        assert_eq!(asset.name, "test.png");
    }

    #[test]
    fn test_asset_register_empty_name() {
        let project_id = ProjectId::new();
        let result = Asset::register(project_id, "", AssetKind::Image);
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_register_whitespace_name() {
        let project_id = ProjectId::new();
        let result = Asset::register(project_id, "   ", AssetKind::Image);
        assert!(result.is_err());
    }

    #[test]
    fn test_asset_with_image_metadata() {
        let project_id = ProjectId::new();
        let asset = Asset::with_image_metadata(
            project_id,
            "hero.png",
            "/projects/test/artifex-assets/image/hero.png",
            512,
            1024,
            65536,
        );

        assert_eq!(asset.name, "hero.png");
        assert_eq!(asset.kind, AssetKind::Image);
        assert_eq!(
            asset.file_path,
            Some("/projects/test/artifex-assets/image/hero.png".to_string())
        );
        assert_eq!(asset.width, Some(512));
        assert_eq!(asset.height, Some(1024));
        assert_eq!(asset.file_size, Some(65536));
        assert!(asset.metadata.is_some());
    }

    #[test]
    fn test_asset_builder_pattern() {
        let project_id = ProjectId::new();
        let asset = Asset::new(project_id, "test.mp3", AssetKind::Audio)
            .with_file_path("/path/to/test.mp3")
            .with_file_size(1024000);

        assert_eq!(asset.kind, AssetKind::Audio);
        assert_eq!(asset.file_path, Some("/path/to/test.mp3".to_string()));
        assert_eq!(asset.file_size, Some(1024000));
    }

    #[test]
    fn test_asset_serde_roundtrip() {
        let project_id = ProjectId::new();
        let asset =
            Asset::with_image_metadata(project_id, "test.png", "/path/test.png", 100, 200, 5000);

        let json = serde_json::to_string(&asset).unwrap();
        let deserialized: Asset = serde_json::from_str(&json).unwrap();

        assert_eq!(asset.id, deserialized.id);
        assert_eq!(asset.name, deserialized.name);
        assert_eq!(asset.kind, deserialized.kind);
        assert_eq!(asset.width, deserialized.width);
        assert_eq!(asset.height, deserialized.height);
    }

    // =========================================================================
    // AnimationMetadata tests
    // =========================================================================

    #[test]
    fn test_animation_metadata_creation() {
        let meta = AnimationMetadata::new(
            "walk_cycle".to_string(),
            vec![
                "frame1".to_string(),
                "frame2".to_string(),
                "frame3".to_string(),
            ],
            vec![100, 200, 150],
            true,
            Some(10),
        )
        .unwrap();

        assert_eq!(meta.name, "walk_cycle");
        assert_eq!(meta.frame_asset_ids.len(), 3);
        assert_eq!(meta.frame_durations_ms, vec![100, 200, 150]);
        assert!(meta.loop_animation);
        assert_eq!(meta.total_duration_ms, 450); // SC-2a: total is sum
        assert_eq!(meta.default_fps, Some(10));
    }

    #[test]
    fn test_animation_metadata_total_duration() {
        // SC-2a: Model stores derived total duration
        let meta = AnimationMetadata::new(
            "test".to_string(),
            vec!["a".to_string(), "b".to_string(), "c".to_string()],
            vec![100, 200, 150],
            false,
            None,
        )
        .unwrap();

        assert_eq!(meta.total_duration_ms, 450);
    }

    #[test]
    fn test_animation_metadata_empty_frames_error() {
        // SC-6: Error — empty frame list
        let result = AnimationMetadata::new("test".to_string(), vec![], vec![], false, None);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Animation must have at least one frame"
        );
    }

    #[test]
    fn test_animation_metadata_mismatched_lengths_error() {
        let result = AnimationMetadata::new(
            "test".to_string(),
            vec!["a".to_string(), "b".to_string()],
            vec![100], // Only one duration for two frames
            false,
            None,
        );
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "frame_asset_ids and frame_durations_ms must have the same length"
        );
    }

    #[test]
    fn test_animation_metadata_zero_duration_error() {
        let result = AnimationMetadata::new(
            "test".to_string(),
            vec!["a".to_string(), "b".to_string()],
            vec![100, 0], // Zero duration is invalid
            false,
            None,
        );
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "All frame durations must be positive");
    }

    #[test]
    fn test_animation_metadata_with_uniform_fps() {
        // SC-1: Create animation from sliced frames at 12 fps = 83ms each
        let meta = AnimationMetadata::with_uniform_fps(
            "idle".to_string(),
            vec![
                "frame0".to_string(),
                "frame1".to_string(),
                "frame2".to_string(),
                "frame3".to_string(),
            ],
            12,
            true,
        )
        .unwrap();

        assert_eq!(meta.name, "idle");
        assert_eq!(meta.frame_asset_ids.len(), 4);
        assert_eq!(meta.frame_durations_ms, vec![83, 83, 83, 83]); // 1000/12 ≈ 83
        assert!(meta.loop_animation);
        assert_eq!(meta.default_fps, Some(12));
    }

    #[test]
    fn test_animation_metadata_serde_roundtrip() {
        let meta = AnimationMetadata::new(
            "run".to_string(),
            vec!["frame1".to_string(), "frame2".to_string()],
            vec![50, 50],
            true,
            Some(20),
        )
        .unwrap();

        let json = serde_json::to_string(&meta).unwrap();
        let deserialized: AnimationMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(meta.name, deserialized.name);
        assert_eq!(meta.frame_asset_ids, deserialized.frame_asset_ids);
        assert_eq!(meta.frame_durations_ms, deserialized.frame_durations_ms);
        assert_eq!(meta.loop_animation, deserialized.loop_animation);
        assert_eq!(meta.total_duration_ms, deserialized.total_duration_ms);
        assert_eq!(meta.default_fps, deserialized.default_fps);
    }

    #[test]
    fn test_animation_metadata_json_format() {
        let meta = AnimationMetadata::new(
            "test".to_string(),
            vec!["id1".to_string(), "id2".to_string()],
            vec![100, 200],
            true,
            None,
        )
        .unwrap();

        let json = serde_json::to_string(&meta).unwrap();
        // Verify camelCase field names
        assert!(json.contains("\"name\":"));
        assert!(json.contains("\"frameAssetIds\":"));
        assert!(json.contains("\"frameDurationsMs\":"));
        assert!(json.contains("\"loopAnimation\":"));
        assert!(json.contains("\"totalDurationMs\":"));
        assert!(!json.contains("\"loop_animation\":")); // Should be serialized as camelCase
        assert!(!json.contains("\"defaultFps\":")); // Should be skipped when None
    }
}
