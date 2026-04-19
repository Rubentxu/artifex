// Identity API - IPC bindings for user profile, tier, and usage tracking
import { invoke } from '@tauri-apps/api/core';
import type { UserProfileDto, UsageEntry, QuotaResult, Tier } from '$lib/types';

/**
 * Gets the current user profile.
 */
export async function getCurrentUser(): Promise<UserProfileDto> {
  return invoke<UserProfileDto>('get_current_user');
}

/**
 * Updates the user profile.
 */
export async function updateProfile(
  displayName?: string | null,
  email?: string | null,
  avatarPath?: string | null
): Promise<UserProfileDto> {
  return invoke<UserProfileDto>('update_profile', {
    displayName: displayName ?? null,
    email: email ?? null,
    avatarPath: avatarPath ?? null,
  });
}

/**
 * Sets the tier (for license activation).
 */
export async function setTier(tier: Tier): Promise<Tier> {
  return invoke<Tier>('set_tier', { tier });
}

/**
 * Gets usage statistics, optionally filtered by operation type.
 */
export async function getUsage(operationType?: string): Promise<UsageEntry[]> {
  return invoke<UsageEntry[]>('get_usage', { operationType: operationType ?? null });
}

/**
 * Checks quota for an operation without incrementing.
 */
export async function checkQuota(operationType: string): Promise<QuotaResult> {
  return invoke<QuotaResult>('check_quota', { operationType });
}
