//! Identity service implementing user profile, tier, and quota operations.

use std::sync::Arc;

use artifex_shared_kernel::ArtifexError;
use crate::identity::dto::{TierDto, UsageEntryDto, UserProfileDto};
use crate::identity::repository::{self, IdentityRepository};

/// Result of a quota check operation.
#[derive(Debug, Clone)]
pub enum QuotaResult {
    /// User is allowed to proceed, with remaining quota.
    Allow { remaining: u32 },
    /// User has exceeded their quota.
    Exceeded { limit: u32, period: String },
}

/// Identity service providing profile, tier, and quota management.
pub struct IdentityService {
    repo: Arc<dyn IdentityRepository>,
}

impl IdentityService {
    /// Creates a new IdentityService with the given repository.
    pub fn new(repo: Arc<dyn IdentityRepository>) -> Self {
        Self { repo }
    }

    /// Gets the current user profile.
    pub async fn get_profile(&self) -> Result<UserProfileDto, ArtifexError> {
        self.repo
            .get_profile()
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))?
            .ok_or_else(|| ArtifexError::NotFound("User profile not found".to_string()))
    }

    /// Updates the user profile.
    pub async fn update_profile(
        &self,
        display_name: Option<String>,
        email: Option<String>,
        avatar_path: Option<String>,
    ) -> Result<UserProfileDto, ArtifexError> {
        self.repo
            .update_profile(display_name, email, avatar_path)
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))
    }

    /// Gets the current tier.
    pub async fn get_tier(&self) -> Result<artifex_shared_kernel::Tier, ArtifexError> {
        let tier_dto = self
            .repo
            .get_tier()
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))?;

        Ok(tier_dto.into())
    }

    /// Sets the tier (admin operation for license activation).
    pub async fn set_tier(&self, tier: artifex_shared_kernel::Tier) -> Result<(), ArtifexError> {
        let tier_dto: TierDto = tier.into();
        self.repo
            .set_tier(tier_dto)
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))
    }

    /// Checks quota for an operation type without incrementing.
    pub async fn check_quota(
        &self,
        operation_type: &str,
    ) -> Result<QuotaResult, ArtifexError> {
        // Pro users have unlimited quota
        if self.get_tier().await?.is_pro() {
            return Ok(QuotaResult::Allow {
                remaining: u32::MAX,
            });
        }

        // Check if this operation is quota-gated
        if !repository::SqliteIdentityRepository::is_quota_gated(operation_type) {
            return Ok(QuotaResult::Allow {
                remaining: u32::MAX,
            });
        }

        let period = repository::SqliteIdentityRepository::current_period();
        let limit = repository::SqliteIdentityRepository::monthly_limit(operation_type);
        let current_count = self
            .repo
            .get_usage(operation_type, &period)
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))?
            .unwrap_or(0);

        if current_count >= limit {
            Ok(QuotaResult::Exceeded {
                limit,
                period,
            })
        } else {
            Ok(QuotaResult::Allow {
                remaining: limit - current_count,
            })
        }
    }

    /// Increments usage counter for an operation type.
    pub async fn increment_usage(
        &self,
        operation_type: &str,
    ) -> Result<(), ArtifexError> {
        let period = repository::SqliteIdentityRepository::current_period();
        self.repo
            .increment_usage(operation_type, &period)
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))
    }

    /// Atomically checks quota and increments usage if allowed.
    /// Returns QuotaResult indicating whether the operation is allowed.
    pub async fn check_and_increment_quota(
        &self,
        operation_type: &str,
    ) -> Result<QuotaResult, ArtifexError> {
        // Pro users have unlimited quota
        if self.get_tier().await?.is_pro() {
            // Still increment usage for Pro tier (for tracking)
            self.increment_usage(operation_type).await.ok();
            return Ok(QuotaResult::Allow {
                remaining: u32::MAX,
            });
        }

        // Check if this operation is quota-gated
        if !repository::SqliteIdentityRepository::is_quota_gated(operation_type) {
            // Not quota-gated, just increment
            self.increment_usage(operation_type).await?;
            return Ok(QuotaResult::Allow {
                remaining: u32::MAX,
            });
        }

        let period = repository::SqliteIdentityRepository::current_period();
        let limit = repository::SqliteIdentityRepository::monthly_limit(operation_type);

        // First check if we have quota
        let current_count = self
            .repo
            .get_usage(operation_type, &period)
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))?
            .unwrap_or(0);

        if current_count >= limit {
            return Ok(QuotaResult::Exceeded {
                limit,
                period,
            });
        }

        // We have quota, increment
        self.increment_usage(operation_type).await?;

        Ok(QuotaResult::Allow {
            remaining: limit - current_count - 1,
        })
    }

    /// Gets all usage entries.
    pub async fn get_usage(&self) -> Result<Vec<UsageEntryDto>, ArtifexError> {
        self.repo
            .get_all_usage()
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))
    }

    /// Gets usage entries filtered by operation type.
    pub async fn get_usage_by_type(
        &self,
        operation_type: &str,
    ) -> Result<Vec<UsageEntryDto>, ArtifexError> {
        self.repo
            .get_usage_by_type(operation_type)
            .await
            .map_err(|e| ArtifexError::Internal(e.to_string()))
    }
}
