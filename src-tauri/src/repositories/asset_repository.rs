//! SQLite implementation of the asset repository.

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{FromRow, SqlitePool};

use artifex_asset_management::{Asset, AssetKind, AssetRepository};
use artifex_shared_kernel::{is_unique_violation, ArtifexError, AssetId, ProjectId, Timestamp};

/// SQLite row representation of an asset.
#[derive(FromRow)]
struct AssetRow {
    id: String,
    project_id: String,
    name: String,
    kind: String,
    file_path: Option<String>,
    metadata: Option<String>,
    file_size: Option<i64>,
    width: Option<i32>,
    height: Option<i32>,
    created_at: String,
    updated_at: String,
}

/// SQLite-backed asset repository.
pub struct SqliteAssetRepository {
    pool: SqlitePool,
}

impl SqliteAssetRepository {
    /// Creates a new SqliteAssetRepository.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AssetRepository for SqliteAssetRepository {
    async fn create(&self, asset: &Asset) -> Result<Asset, ArtifexError> {
        let metadata_json = asset
            .metadata
            .as_ref()
            .map(|m| serde_json::to_string(m).unwrap_or_else(|_| "{}".to_string()));

        let now = Timestamp::now();
        let result = sqlx::query(
            r#"INSERT INTO assets (id, project_id, name, kind, file_path, metadata, file_size, width, height, created_at, updated_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(asset.id.into_uuid().to_string())
        .bind(asset.project_id.into_uuid().to_string())
        .bind(&asset.name)
        .bind(asset.kind.as_str())
        .bind(&asset.file_path)
        .bind(&metadata_json)
        .bind(asset.file_size.map(|s| s as i64))
        .bind(asset.width.map(|w| w as i32))
        .bind(asset.height.map(|h| h as i32))
        .bind(asset.created_at.to_string())
        .bind(now.to_string())
        .execute(&self.pool)
        .await;

        match result {
            Ok(_) => Ok(asset.clone()),
            Err(e) if is_unique_violation(&e) => {
                Err(ArtifexError::Internal(format!(
                    "Duplicate asset id (UUID collision): {}",
                    e
                )))
            }
            Err(e) => Err(ArtifexError::IoError(e.to_string())),
        }
    }

    async fn find_by_id(&self, id: &AssetId) -> Result<Option<Asset>, ArtifexError> {
        let row: Option<AssetRow> = sqlx::query_as(
            r#"SELECT id, project_id, name, kind, file_path, metadata, file_size, width, height, created_at, updated_at
               FROM assets WHERE id = ?"#,
        )
        .bind(id.into_uuid().to_string())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        row.map(|r| row_to_asset(&r)).transpose()
    }

    async fn find_by_project(&self, project_id: &ProjectId) -> Result<Vec<Asset>, ArtifexError> {
        let rows: Vec<AssetRow> = sqlx::query_as(
            r#"SELECT id, project_id, name, kind, file_path, metadata, file_size, width, height, created_at, updated_at
               FROM assets WHERE project_id = ? ORDER BY created_at DESC"#,
        )
        .bind(project_id.into_uuid().to_string())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        let mut assets = Vec::with_capacity(rows.len());
        for row in rows {
            assets.push(row_to_asset(&row)?);
        }
        Ok(assets)
    }

    async fn find_by_kind(
        &self,
        project_id: &ProjectId,
        kind: &AssetKind,
    ) -> Result<Vec<Asset>, ArtifexError> {
        let rows: Vec<AssetRow> = sqlx::query_as(
            r#"SELECT id, project_id, name, kind, file_path, metadata, file_size, width, height, created_at, updated_at
               FROM assets WHERE project_id = ? AND kind = ? ORDER BY created_at DESC"#,
        )
        .bind(project_id.into_uuid().to_string())
        .bind(kind.as_str())
        .fetch_all(&self.pool)
        .await
        .map_err(|e| ArtifexError::IoError(e.to_string()))?;

        let mut assets = Vec::with_capacity(rows.len());
        for row in rows {
            assets.push(row_to_asset(&row)?);
        }
        Ok(assets)
    }

    async fn delete(&self, id: &AssetId) -> Result<(), ArtifexError> {
        let result = sqlx::query("DELETE FROM assets WHERE id = ?")
            .bind(id.into_uuid().to_string())
            .execute(&self.pool)
            .await;

        match result {
            Ok(affected) if affected.rows_affected() == 0 => {
                Err(ArtifexError::NotFound(format!("Asset {} not found", id.into_uuid())))
            }
            Ok(_) => Ok(()),
            Err(e) => Err(ArtifexError::IoError(e.to_string())),
        }
    }
}

/// Converts a database row to an Asset domain object.
fn row_to_asset(row: &AssetRow) -> Result<Asset, ArtifexError> {
    let id_uuid = uuid::Uuid::parse_str(&row.id)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid asset id: {}", e)))?;
    let id = AssetId::from_uuid(id_uuid);

    let project_uuid = uuid::Uuid::parse_str(&row.project_id)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid project id: {}", e)))?;
    let project_id = ProjectId::from_uuid(project_uuid);

    let kind = AssetKind::from_str(&row.kind)
        .ok_or_else(|| {
            ArtifexError::ValidationError(format!("Unknown asset kind: {}", row.kind))
        })?;

    let metadata = row.metadata.as_ref().map(|m| {
        serde_json::from_str(m).unwrap_or_else(|_| serde_json::Value::Object(Default::default()))
    });

    let created_at = parse_rfc3339(&row.created_at)?;

    Ok(Asset {
        id,
        project_id,
        name: row.name.clone(),
        kind,
        file_path: row.file_path.clone(),
        metadata,
        file_size: row.file_size.map(|s| s as u64),
        width: row.width.map(|w| w as u32),
        height: row.height.map(|h| h as u32),
        created_at,
    })
}

/// Parses an RFC3339 timestamp string into a Timestamp.
fn parse_rfc3339(s: &str) -> Result<Timestamp, ArtifexError> {
    let dt = DateTime::parse_from_rfc3339(s)
        .map_err(|e| ArtifexError::ValidationError(format!("Invalid timestamp: {}", e)))?;
    Ok(Timestamp::from_datetime(dt.with_timezone(&Utc)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use artifex_shared_kernel::ProjectId;

    #[test]
    fn test_row_to_asset_parses_correctly() {
        let row = AssetRow {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: uuid::Uuid::new_v4().to_string(),
            name: "test.png".to_string(),
            kind: "image".to_string(),
            file_path: Some("/path/to/test.png".to_string()),
            metadata: Some(r#"{"width": 512, "height": 1024}"#.to_string()),
            file_size: Some(65536),
            width: Some(512),
            height: Some(1024),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let asset = row_to_asset(&row).unwrap();
        assert_eq!(asset.name, "test.png");
        assert_eq!(asset.kind, AssetKind::Image);
        assert_eq!(asset.file_path, Some("/path/to/test.png".to_string()));
        assert_eq!(asset.file_size, Some(65536));
        assert_eq!(asset.width, Some(512));
        assert_eq!(asset.height, Some(1024));
    }

    #[test]
    fn test_row_to_asset_with_minimal_fields() {
        let row = AssetRow {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: uuid::Uuid::new_v4().to_string(),
            name: "minimal.txt".to_string(),
            kind: "other".to_string(),
            file_path: None,
            metadata: None,
            file_size: None,
            width: None,
            height: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let asset = row_to_asset(&row).unwrap();
        assert_eq!(asset.name, "minimal.txt");
        assert_eq!(asset.kind, AssetKind::Other);
        assert!(asset.file_path.is_none());
        assert!(asset.metadata.is_none());
        assert!(asset.file_size.is_none());
    }

    #[test]
    fn test_invalid_asset_kind_returns_error() {
        let row = AssetRow {
            id: uuid::Uuid::new_v4().to_string(),
            project_id: uuid::Uuid::new_v4().to_string(),
            name: "test.txt".to_string(),
            kind: "invalid_kind".to_string(),
            file_path: None,
            metadata: None,
            file_size: None,
            width: None,
            height: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let result = row_to_asset(&row);
        assert!(result.is_err());
    }
}
