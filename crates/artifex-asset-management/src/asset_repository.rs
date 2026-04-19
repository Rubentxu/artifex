//! Asset repository trait.

use async_trait::async_trait;

use artifex_shared_kernel::{ArtifexError, AssetId, ProjectId};

use super::asset::{Asset, AssetKind};

/// Repository trait for asset persistence.
///
/// Concrete implementations will be added in the application layer.
#[async_trait]
pub trait AssetRepository: Send + Sync {
    /// Creates a new asset.
    async fn create(&self, asset: &Asset) -> Result<Asset, ArtifexError>;

    /// Finds an asset by its ID.
    async fn find_by_id(&self, id: &AssetId) -> Result<Option<Asset>, ArtifexError>;

    /// Finds all assets for a given project.
    async fn find_by_project(&self, project_id: &ProjectId) -> Result<Vec<Asset>, ArtifexError>;

    /// Finds all assets of a specific kind within a project.
    async fn find_by_kind(
        &self,
        project_id: &ProjectId,
        kind: &AssetKind,
    ) -> Result<Vec<Asset>, ArtifexError>;

    /// Finds all assets with a specific tag within a project.
    async fn find_by_tag(
        &self,
        project_id: &ProjectId,
        tag: &str,
    ) -> Result<Vec<Asset>, ArtifexError>;

    /// Finds all assets in a specific collection within a project.
    async fn find_by_collection(
        &self,
        project_id: &ProjectId,
        collection_id: &str,
    ) -> Result<Vec<Asset>, ArtifexError>;

    /// Updates the tags for an asset.
    async fn update_tags(&self, id: &AssetId, tags: &[String]) -> Result<(), ArtifexError>;

    /// Updates the collection ID for an asset.
    async fn update_collection(
        &self,
        id: &AssetId,
        collection_id: Option<&str>,
    ) -> Result<(), ArtifexError>;

    /// Deletes an asset by its ID.
    async fn delete(&self, id: &AssetId) -> Result<(), ArtifexError>;
}
