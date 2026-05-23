<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    icon?: 'folder' | 'search' | 'error' | 'info';
    title?: string;
    description?: string;
    children?: Snippet;
  }

  let { icon = 'folder', title = '', description = '', children }: Props = $props();

  const icons: Record<string, string> = {
    folder: 'M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z',
    search: 'M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z',
    error: 'M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z',
    info: 'M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z',
  };
</script>

<div class="flex flex-col items-center justify-center py-12 px-4 text-center">
  <div class="w-16 h-16 mb-4 text-[var(--color-cyan)]" style="filter: drop-shadow(var(--glow-cyan));">
    <svg class="w-full h-full" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d={icons[icon]} />
    </svg>
  </div>
  <h3 class="text-lg font-medium text-[var(--color-text)] mb-1">{title}</h3>
  {#if description}
    <p class="text-sm text-[var(--color-text-muted)] mb-4">{description}</p>
  {/if}
  {#if children}
    <div class="mt-2">
      {@render children()}
    </div>
  {/if}
</div>
