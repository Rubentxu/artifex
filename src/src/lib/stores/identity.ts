// Identity store - reactive state for user profile, tier, and usage
import { writable, derived } from 'svelte/store';
import type { UserProfileDto, UsageEntry, Tier } from '$lib/types';
import * as identityApi from '$lib/api/identity';

interface IdentityState {
  user: UserProfileDto | null;
  tier: Tier;
  usage: UsageEntry[];
  loading: boolean;
  error: string | null;
}

function createIdentityStore() {
  const { subscribe, set, update } = writable<IdentityState>({
    user: null,
    tier: 'free',
    usage: [],
    loading: false,
    error: null,
  });

  return {
    subscribe,

    /**
     * Loads the current user profile and tier.
     */
    async loadIdentity() {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const user = await identityApi.getCurrentUser();
        update(s => ({
          ...s,
          user,
          tier: user.tier,
          loading: false,
        }));
      } catch (e) {
        update(s => ({
          ...s,
          error: e instanceof Error ? e.message : String(e),
          loading: false,
        }));
      }
    },

    /**
     * Updates the user profile.
     */
    async updateProfile(displayName?: string, email?: string, avatarPath?: string) {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const user = await identityApi.updateProfile(displayName, email, avatarPath);
        update(s => ({
          ...s,
          user,
          loading: false,
        }));
        return user;
      } catch (e) {
        update(s => ({
          ...s,
          error: e instanceof Error ? e.message : String(e),
          loading: false,
        }));
        throw e;
      }
    },

    /**
     * Sets the tier (for license activation).
     */
    async setTier(tier: Tier) {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        await identityApi.setTier(tier);
        update(s => ({
          ...s,
          tier,
          loading: false,
        }));
      } catch (e) {
        update(s => ({
          ...s,
          error: e instanceof Error ? e.message : String(e),
          loading: false,
        }));
        throw e;
      }
    },

    /**
     * Loads usage statistics.
     */
    async loadUsage(operationType?: string) {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const usage = await identityApi.getUsage(operationType);
        update(s => ({
          ...s,
          usage,
          loading: false,
        }));
      } catch (e) {
        update(s => ({
          ...s,
          error: e instanceof Error ? e.message : String(e),
          loading: false,
        }));
      }
    },

    /**
     * Resets the store.
     */
    reset() {
      set({
        user: null,
        tier: 'free',
        usage: [],
        loading: false,
        error: null,
      });
    },
  };
}

export const identityStore = createIdentityStore();

// Derived stores for convenience
export const currentTier = derived(identityStore, ($state) => $state.tier);
export const currentUser = derived(identityStore, ($state) => $state.user);
export const isPro = derived(identityStore, ($state) => $state.tier === 'pro');
