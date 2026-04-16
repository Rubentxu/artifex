<script lang="ts">
  import '../app.css';
  import AppShell from '$lib/components/AppShell.svelte';
  import { initStores } from '$lib/stores/ui';
  import { onMount } from 'svelte';

  let { children } = $props();
  let initialized = $state(false);

  onMount(async () => {
    await initStores();
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
