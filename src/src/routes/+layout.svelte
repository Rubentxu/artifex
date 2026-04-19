<script lang="ts">
  import '../app.css';
  import AppShell from '$lib/components/AppShell.svelte';
  import { initStores } from '$lib/stores/ui';
  import { onMount } from 'svelte';

  let { children } = $props();
  let initialized = $state(false);

  onMount(async () => {
    await initStores();
    // @ts-ignore — __ARTIFEX_E2E__ is injected by vite.config.ts define
    if (import.meta.env.DEV || __ARTIFEX_E2E__) {
      const { initDebugHarness } = await import('$lib/debug/harness');
      initDebugHarness();
    }
    initialized = true;
  });
</script>

{#if initialized}
  <AppShell>
    {@render children()}
  </AppShell>
{:else}
  <!-- Loading state -->
  <div class="h-screen w-screen flex items-center justify-center bg-[var(--color-canvas)]">
    <div class="text-[var(--color-text-muted)]">Loading...</div>
  </div>
{/if}
