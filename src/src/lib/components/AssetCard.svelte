<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { AssetResponse } from '$lib/types/asset';
  import Badge from '$lib/components/ui/Badge.svelte';

  interface Props {
    asset: AssetResponse;
    selected: boolean;
    onclick: () => void;
  }

  let { asset, selected, onclick }: Props = $props();

  function formatFileSize(bytes: number | null): string {
    if (bytes === null) return 'Unknown';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function isImageAsset(kind: string): boolean {
    return kind === 'Image' || kind === 'Sprite' || kind === 'Tileset';
  }

  function isAudioAsset(kind: string): boolean {
    return kind === 'Audio' || kind === 'Voice';
  }

  function getImageSrc(filePath: string | null): string | null {
    if (!filePath) return null;
    return convertFileSrc(filePath);
  }

  function formatDuration(secs: number | null | undefined): string {
    if (secs === null || secs === undefined) return '--:--';
    const mins = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${mins}:${s.toString().padStart(2, '0')}`;
  }
</script>

<button
  {onclick}
  class="w-full text-left p-3 rounded-sm border transition-all duration-150 hover:border-[var(--color-neon)]/50 hover:shadow-[var(--glow-neon)] bg-[var(--color-panel)] hover:bg-[var(--color-panel)]/80 border-[var(--color-surface)]"
  class:ring-2={selected}
  class:ring-[var(--color-neon)]={selected}
>
  <!-- Thumbnail area -->
  <div class="w-full aspect-video rounded-sm bg-[var(--color-surface)] flex items-center justify-center mb-2 overflow-hidden">
    {#if asset.file_path && isImageAsset(asset.kind)}
      {@const src = getImageSrc(asset.file_path)}
      {#if src}
        <img
          {src}
          alt={asset.name}
          class="w-full h-full object-cover"
          loading="lazy"
        />
      {:else}
        <div class="text-xs text-[var(--color-text-muted)] truncate px-1">
          {asset.width}x{asset.height}
        </div>
      {/if}
    {:else if asset.file_path && isAudioAsset(asset.kind)}
      {@const src = getImageSrc(asset.file_path)}
      {#if src}
        <audio
          {src}
          controls
          class="w-full h-12"
          preload="metadata"
        >
          <track kind="captions" />
        </audio>
      {:else}
        <svg class="w-8 h-8 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
        </svg>
      {/if}
    {:else}
      <svg class="w-8 h-8 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        {#if asset.kind === 'Audio' || asset.kind === 'Voice'}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
        {:else if asset.kind === 'Video'}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 10l4.553-2.276A1 1 0 0121 8.618v6.764a1 1 0 01-1.447.894L15 14M5 18h8a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
        {:else}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
        {/if}
      </svg>
    {/if}
  </div>

  <!-- Name -->
  <div class="font-medium text-sm truncate mb-1" title={asset.name}>
    {asset.name}
  </div>

  <!-- Kind badge and size -->
  <div class="flex items-center justify-between">
    <Badge variant={asset.kind.toLowerCase()}>{asset.kind}</Badge>
    {#if isAudioAsset(asset.kind) && asset.duration_secs}
      <span class="text-xs text-[var(--color-text-muted)]">
        {formatDuration(asset.duration_secs)}
      </span>
    {:else if asset.file_size}
      <span class="text-xs text-[var(--color-text-muted)]">
        {formatFileSize(asset.file_size)}
      </span>
    {/if}
  </div>
</button>
