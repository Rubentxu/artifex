//! Artifex Asset Management
//!
//! Project management domain types and repository traits.

pub mod asset;
pub mod asset_repository;
pub mod code_engine;
pub mod collection;
pub mod project;
pub mod project_name;
pub mod repository;

// Re-exports
pub use asset::{Asset, AssetKind};
pub use asset_repository::AssetRepository;
pub use code_engine::CodeEngine;
pub use collection::Collection;
pub use project::{Project, ProjectStatus};
pub use project_name::ProjectName;
pub use repository::ProjectRepository;
