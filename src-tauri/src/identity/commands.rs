//! IPC command handlers for identity operations.

use tauri::State;

use crate::identity::dto::{QuotaResultDto, TierDto, UsageEntryDto, UserProfileDto};
use crate::state::AppState;

/// Gets the current user profile.
#[tauri::command]
pub async fn get_current_user(
    state: State<'_, AppState>,
) -> Result<UserProfileDto, String> {
    state
        .identity_service
        .get_profile()
        .await
        .map_err(|e| e.to_string())
}

/// Updates the user profile.
#[tauri::command]
pub async fn update_profile(
    state: State<'_, AppState>,
    display_name: Option<String>,
    email: Option<String>,
    avatar_path: Option<String>,
) -> Result<UserProfileDto, String> {
    state
        .identity_service
        .update_profile(display_name, email, avatar_path)
        .await
        .map_err(|e| e.to_string())
}

/// Sets the tier (for license key activation).
#[tauri::command]
pub async fn set_tier(
    state: State<'_, AppState>,
    tier: String,
) -> Result<TierDto, String> {
    let tier_parsed = tier.parse::<artifex_shared_kernel::Tier>()
        .map_err(|e: String| e)?;
    state
        .identity_service
        .set_tier(tier_parsed)
        .await
        .map_err(|e| e.to_string())?;
    Ok(tier_parsed.into())
}

/// Gets usage statistics, optionally filtered by operation type.
#[tauri::command]
pub async fn get_usage(
    state: State<'_, AppState>,
    operation_type: Option<String>,
) -> Result<Vec<UsageEntryDto>, String> {
    match operation_type {
        Some(op_type) => {
            state
                .identity_service
                .get_usage_by_type(&op_type)
                .await
                .map_err(|e| e.to_string())
        }
        None => {
            state
                .identity_service
                .get_usage()
                .await
                .map_err(|e| e.to_string())
        }
    }
}

/// Checks quota for an operation without incrementing.
#[tauri::command]
pub async fn check_quota(
    state: State<'_, AppState>,
    operation_type: String,
) -> Result<QuotaResultDto, String> {
    use crate::identity::service::QuotaResult;

    match state
        .identity_service
        .check_quota(&operation_type)
        .await
        .map_err(|e| e.to_string())?
    {
        QuotaResult::Allow { remaining } => {
            let limit = crate::identity::repository::SqliteIdentityRepository::monthly_limit(&operation_type);
            Ok(QuotaResultDto {
                allowed: true,
                remaining,
                limit,
                period: crate::identity::repository::SqliteIdentityRepository::current_period(),
            })
        }
        QuotaResult::Exceeded { limit, period } => Ok(QuotaResultDto {
            allowed: false,
            remaining: 0,
            limit,
            period,
        }),
    }
}
