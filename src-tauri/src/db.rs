//! Database initialization and connection pool management.

use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};
use std::path::Path;

/// Initializes a SQLite connection pool at the given path and runs migrations.
pub async fn init_db_pool<P: AsRef<Path>>(db_path: P) -> Result<SqlitePool, sqlx::Error> {
    let db_url = format!(
        "sqlite:{}?mode=rwc",
        db_path.as_ref().display()
    );
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&db_url)
        .await?;

    // Run migrations from the migrations directory
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

/// Row type for user_profile table queries.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserProfileRow {
    pub id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub avatar_path: Option<String>,
    pub tier: String,
    pub license_key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

impl From<UserProfileRow> for crate::identity::dto::UserProfileDto {
    fn from(row: UserProfileRow) -> Self {
        let tier = match row.tier.as_str() {
            "pro" => crate::identity::dto::TierDto::Pro,
            _ => crate::identity::dto::TierDto::Free,
        };
        Self {
            id: row.id,
            display_name: row.display_name,
            email: row.email,
            avatar_path: row.avatar_path,
            tier,
            license_key: row.license_key,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

/// Row type for usage_counters table queries.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UsageCounterRow {
    pub id: String,
    pub user_id: String,
    pub operation_type: String,
    pub period: String,
    pub count: i64,
    pub updated_at: String,
}

impl From<UsageCounterRow> for crate::identity::dto::UsageEntryDto {
    fn from(row: UsageCounterRow) -> Self {
        let limit = crate::identity::repository::SqliteIdentityRepository::monthly_limit(&row.operation_type);
        let count = row.count as u32;
        let remaining = if limit == u32::MAX {
            u32::MAX
        } else {
            limit.saturating_sub(count)
        };
        Self {
            operation_type: row.operation_type,
            period: row.period,
            count,
            limit,
            remaining,
        }
    }
}
