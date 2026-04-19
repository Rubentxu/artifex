//! Data Transfer Objects for identity module.
//!
//! All DTOs use snake_case for JSON serialization compatibility with TypeScript.

use serde::{Deserialize, Serialize};

/// User profile DTO returned to frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserProfileDto {
    pub id: String,
    pub display_name: String,
    pub email: Option<String>,
    pub avatar_path: Option<String>,
    pub tier: TierDto,
    pub license_key: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// Tier representation for frontend (snake_case to match TS convention).
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TierDto {
    Free,
    Pro,
}

impl From<artifex_shared_kernel::Tier> for TierDto {
    fn from(tier: artifex_shared_kernel::Tier) -> Self {
        match tier {
            artifex_shared_kernel::Tier::Free => TierDto::Free,
            artifex_shared_kernel::Tier::Pro => TierDto::Pro,
        }
    }
}

impl From<TierDto> for artifex_shared_kernel::Tier {
    fn from(dto: TierDto) -> Self {
        match dto {
            TierDto::Free => artifex_shared_kernel::Tier::Free,
            TierDto::Pro => artifex_shared_kernel::Tier::Pro,
        }
    }
}

/// Usage entry DTO for quota display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UsageEntryDto {
    pub operation_type: String,
    pub period: String,
    pub count: u32,
    pub limit: u32,
    pub remaining: u32,
}

/// Result of a quota check.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct QuotaResultDto {
    pub allowed: bool,
    pub remaining: u32,
    pub limit: u32,
    pub period: String,
}
