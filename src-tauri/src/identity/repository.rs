//! Identity repository for SQLite persistence.

use sqlx::{Pool, Sqlite};
use std::sync::Arc;

use crate::db;
use crate::identity::dto::{TierDto, UsageEntryDto, UserProfileDto};

/// Free tier image generation monthly limit.
pub const IMAGE_GEN_MONTHLY_LIMIT: u32 = 50;

/// Free tier audio generation monthly limit.
pub const AUDIO_GEN_MONTHLY_LIMIT: u32 = 20;

/// Repository trait for identity operations.
#[async_trait::async_trait]
pub trait IdentityRepository: Send + Sync {
    /// Gets the user profile.
    async fn get_profile(&self) -> Result<Option<UserProfileDto>, sqlx::Error>;

    /// Updates the user profile.
    async fn update_profile(
        &self,
        display_name: Option<String>,
        email: Option<String>,
        avatar_path: Option<String>,
    ) -> Result<UserProfileDto, sqlx::Error>;

    /// Gets the current tier.
    async fn get_tier(&self) -> Result<TierDto, sqlx::Error>;

    /// Sets the tier.
    async fn set_tier(&self, tier: TierDto) -> Result<(), sqlx::Error>;

    /// Gets usage counter for an operation type in a period.
    async fn get_usage(
        &self,
        operation_type: &str,
        period: &str,
    ) -> Result<Option<u32>, sqlx::Error>;

    /// Gets all usage entries.
    async fn get_all_usage(&self) -> Result<Vec<UsageEntryDto>, sqlx::Error>;

    /// Gets usage entries filtered by operation type.
    async fn get_usage_by_type(
        &self,
        operation_type: &str,
    ) -> Result<Vec<UsageEntryDto>, sqlx::Error>;

    /// Increments usage counter for an operation type in a period.
    async fn increment_usage(
        &self,
        operation_type: &str,
        period: &str,
    ) -> Result<(), sqlx::Error>;

    /// Ensures a usage counter row exists (for atomic upsert).
    async fn ensure_usage_row(
        &self,
        operation_type: &str,
        period: &str,
    ) -> Result<(), sqlx::Error>;
}

/// SQLite implementation of IdentityRepository.
pub struct SqliteIdentityRepository {
    pool: Pool<Sqlite>,
}

impl SqliteIdentityRepository {
    /// Creates a new repository with the given pool.
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Gets the current period string (YYYY-MM format).
    pub fn current_period() -> String {
        chrono::Local::now().format("%Y-%m").to_string()
    }

    /// Gets the monthly limit for an operation type.
    pub fn monthly_limit(operation_type: &str) -> u32 {
        match operation_type {
            "image_generate" => IMAGE_GEN_MONTHLY_LIMIT,
            "audio_generate" => AUDIO_GEN_MONTHLY_LIMIT,
            _ => u32::MAX, // Unknown operations are unlimited for Pro
        }
    }

    /// Checks if an operation is quota-gated for Free tier.
    pub fn is_quota_gated(operation_type: &str) -> bool {
        matches!(operation_type, "image_generate" | "audio_generate")
    }
}

#[async_trait::async_trait]
impl IdentityRepository for SqliteIdentityRepository {
    async fn get_profile(&self) -> Result<Option<UserProfileDto>, sqlx::Error> {
        let row = sqlx::query_as::<_, db::UserProfileRow>(
            "SELECT id, display_name, email, avatar_path, tier, license_key, created_at, updated_at FROM user_profile LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into()))
    }

    async fn update_profile(
        &self,
        display_name: Option<String>,
        email: Option<String>,
        avatar_path: Option<String>,
    ) -> Result<UserProfileDto, sqlx::Error> {
        // Build dynamic UPDATE based on provided fields
        let mut updates = Vec::new();
        let mut has_update = false;

        if display_name.is_some() {
            updates.push("display_name = ?");
            has_update = true;
        }
        if email.is_some() {
            updates.push("email = ?");
            has_update = true;
        }
        if avatar_path.is_some() {
            updates.push("avatar_path = ?");
            has_update = true;
        }

        if has_update {
            updates.push("updated_at = datetime('now')");
            let query = format!(
                "UPDATE user_profile SET {} WHERE id = 'default-user'",
                updates.join(", ")
            );

            let mut query_builder = sqlx::query(&query);
            if let Some(ref v) = display_name {
                query_builder = query_builder.bind(v);
            }
            if let Some(ref v) = email {
                query_builder = query_builder.bind(v);
            }
            if let Some(ref v) = avatar_path {
                query_builder = query_builder.bind(v);
            }

            query_builder.execute(&self.pool).await?;
        }

        // Fetch and return updated profile
        self.get_profile()
            .await?
            .ok_or_else(|| sqlx::Error::RowNotFound)
    }

    async fn get_tier(&self) -> Result<TierDto, sqlx::Error> {
        let row = sqlx::query_as::<_, (String,)>(
            "SELECT tier FROM user_profile WHERE id = 'default-user'",
        )
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some((tier_str,)) => match tier_str.as_str() {
                "pro" => Ok(TierDto::Pro),
                _ => Ok(TierDto::Free),
            },
            None => Ok(TierDto::Free),
        }
    }

    async fn set_tier(&self, tier: TierDto) -> Result<(), sqlx::Error> {
        let tier_str = match tier {
            TierDto::Free => "free",
            TierDto::Pro => "pro",
        };

        sqlx::query(
            "UPDATE user_profile SET tier = ?, updated_at = datetime('now') WHERE id = 'default-user'",
        )
        .bind(tier_str)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_usage(
        &self,
        operation_type: &str,
        period: &str,
    ) -> Result<Option<u32>, sqlx::Error> {
        let row = sqlx::query_as::<_, (i64,)>(
            "SELECT count FROM usage_counters WHERE user_id = 'default-user' AND operation_type = ? AND period = ?",
        )
        .bind(operation_type)
        .bind(period)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(count,)| count as u32))
    }

    async fn get_all_usage(&self) -> Result<Vec<UsageEntryDto>, sqlx::Error> {
        let rows = sqlx::query_as::<_, db::UsageCounterRow>(
            "SELECT id, user_id, operation_type, period, count, updated_at FROM usage_counters WHERE user_id = 'default-user' ORDER BY period DESC, operation_type",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn get_usage_by_type(
        &self,
        operation_type: &str,
    ) -> Result<Vec<UsageEntryDto>, sqlx::Error> {
        let rows = sqlx::query_as::<_, db::UsageCounterRow>(
            "SELECT id, user_id, operation_type, period, count, updated_at FROM usage_counters WHERE user_id = 'default-user' AND operation_type = ? ORDER BY period DESC",
        )
        .bind(operation_type)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    async fn increment_usage(
        &self,
        operation_type: &str,
        period: &str,
    ) -> Result<(), sqlx::Error> {
        self.ensure_usage_row(operation_type, period).await?;

        sqlx::query(
            "UPDATE usage_counters SET count = count + 1, updated_at = datetime('now') WHERE user_id = 'default-user' AND operation_type = ? AND period = ?",
        )
        .bind(operation_type)
        .bind(period)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn ensure_usage_row(
        &self,
        operation_type: &str,
        period: &str,
    ) -> Result<(), sqlx::Error> {
        let id = uuid::Uuid::new_v4().to_string();

        sqlx::query(
            "INSERT OR IGNORE INTO usage_counters (id, user_id, operation_type, period, count, updated_at) VALUES (?, 'default-user', ?, ?, 0, datetime('now'))",
        )
        .bind(&id)
        .bind(operation_type)
        .bind(period)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

// Helper to create a repository from a pool
pub fn create_identity_repository(pool: Pool<Sqlite>) -> Arc<dyn IdentityRepository> {
    Arc::new(SqliteIdentityRepository::new(pool))
}
