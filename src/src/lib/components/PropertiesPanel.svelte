<script lang="ts">
  import { propertiesCollapsed, toggleProperties } from '$lib/stores/ui';
  import { selectedProject } from '$lib/stores/project';
  import { selectedAsset } from '$lib/stores/asset';

  interface GenerationMetadata {
    prompt?: string;
    negative_prompt?: string;
    width?: number;
    height?: number;
    steps?: number;
    seed?: number;
    model_id?: string;
    provider?: string;
    format?: string;
  }

  interface Props {
    onRemoveBackground?: (assetId: string) => void;
    onConvertPixelArt?: (assetId: string) => void;
  }

  let { onRemoveBackground, onConvertPixelArt }: Props = $props();

  function isImageAsset(kind: string | undefined): boolean {
    return kind === 'Image' || kind === 'Sprite' || kind === 'Tileset' || kind === 'Material';
  }

  function formatDate(dateStr: string): string {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return dateStr;
    }
  }

  function formatFileSize(bytes: number | null): string {
    if (bytes === null) return 'Unknown';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function getGenerationMetadata(asset: typeof $selectedAsset): GenerationMetadata | null {
    if (!asset?.metadata) return null;
    const m = asset.metadata as Record<string, unknown>;
    // Check if this looks like generation metadata (has prompt, model_id, provider, etc.)
    if (m.prompt || m.model_id || m.provider) {
      return m as GenerationMetadata;
    }
    return null;
  }

  let genMetadata = $derived(getGenerationMetadata($selectedAsset));
</script>

<aside
  class="h-full bg-[var(--color-panel)] border-l border-[var(--color-surface)] flex flex-col transition-all duration-200"
  class:w-72={!$propertiesCollapsed}
  class:w-12={$propertiesCollapsed}
>
  <!-- Header -->
  <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)]">
    {#if !$propertiesCollapsed}
      <h2 class="font-semibold">Properties</h2>
    {/if}
    <button
      onclick={toggleProperties}
      class="p-1.5 rounded hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text-muted)]"
      title={$propertiesCollapsed ? 'Expand properties' : 'Collapse properties'}
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        {#if $propertiesCollapsed}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
        {:else}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
        {/if}
      </svg>
    </button>
  </div>

  {#if !$propertiesCollapsed}
    {#if $selectedAsset}
      <!-- Asset details -->
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <!-- Asset Name -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            Name
          </label>
          <p class="mt-1 font-medium">{$selectedAsset.name}</p>
        </div>

        <!-- Asset Kind -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            Kind
          </label>
          <p class="mt-1">
            <span class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-blue-500/20 text-blue-400">
              {$selectedAsset.kind}
            </span>
          </p>
        </div>

        <!-- File Path -->
        {#if $selectedAsset.file_path}
          <div>
            <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
              File Path
            </label>
            <p class="mt-1 text-sm break-all">{$selectedAsset.file_path}</p>
          </div>
        {/if}

        <!-- File Size -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            File Size
          </label>
          <p class="mt-1 text-sm">{formatFileSize($selectedAsset.file_size)}</p>
        </div>

        <!-- Dimensions -->
        {#if $selectedAsset.width && $selectedAsset.height}
          <div>
            <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
              Dimensions
            </label>
            <p class="mt-1 text-sm">{$selectedAsset.width} x {$selectedAsset.height}</p>
          </div>
        {/if}

        <!-- Generation Metadata (for generated images) -->
        {#if genMetadata}
          <div class="border-t border-[var(--color-surface)] pt-4">
            <label class="text-xs font-semibold text-[var(--color-accent)] uppercase tracking-wider">
              Generation Info
            </label>
            <div class="mt-2 space-y-2">
              {#if genMetadata.prompt}
                <div>
                  <label class="text-xs text-[var(--color-text-muted)]">Prompt</label>
                  <p class="mt-0.5 text-sm truncate" title={genMetadata.prompt}>{genMetadata.prompt}</p>
                </div>
              {/if}
              {#if genMetadata.negative_prompt}
                <div>
                  <label class="text-xs text-[var(--color-text-muted)]">Negative Prompt</label>
                  <p class="mt-0.5 text-sm truncate" title={genMetadata.negative_prompt}>{genMetadata.negative_prompt}</p>
                </div>
              {/if}
              {#if genMetadata.provider}
                <div>
                  <label class="text-xs text-[var(--color-text-muted)]">Provider</label>
                  <p class="mt-0.5 text-sm">{genMetadata.provider}</p>
                </div>
              {/if}
              {#if genMetadata.model_id}
                <div>
                  <label class="text-xs text-[var(--color-text-muted)]">Model</label>
                  <p class="mt-0.5 text-sm">{genMetadata.model_id}</p>
                </div>
              {/if}
              {#if genMetadata.steps}
                <div>
                  <label class="text-xs text-[var(--color-text-muted)]">Steps</label>
                  <p class="mt-0.5 text-sm">{genMetadata.steps}</p>
                </div>
              {/if}
              {#if genMetadata.seed}
                <div>
                  <label class="text-xs text-[var(--color-text-muted)]">Seed</label>
                  <p class="mt-0.5 text-sm">{genMetadata.seed}</p>
                </div>
              {/if}
            </div>
          </div>
        {/if}

        <!-- Created At -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            Created
          </label>
          <p class="mt-1 text-sm">{formatDate($selectedAsset.created_at)}</p>
        </div>

        <!-- Asset ID -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            ID
          </label>
          <p class="mt-1 text-xs font-mono break-all">{$selectedAsset.id}</p>
        </div>

        <!-- Image Actions -->
        {#if isImageAsset($selectedAsset.kind)}
          <div class="border-t border-[var(--color-surface)] pt-4 space-y-2">
            <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
              Actions
            </label>
            <div class="flex flex-col gap-2">
              {#if onRemoveBackground}
                <button
                  onclick={() => onRemoveBackground($selectedAsset.id)}
                  class="w-full px-3 py-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors text-sm font-medium text-left flex items-center gap-2"
                >
                  <svg class="w-4 h-4 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
                  </svg>
                  Remove Background
                </button>
              {/if}
              {#if onConvertPixelArt}
                <button
                  onclick={() => onConvertPixelArt($selectedAsset.id)}
                  class="w-full px-3 py-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors text-sm font-medium text-left flex items-center gap-2"
                >
                  <svg class="w-4 h-4 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
                  </svg>
                  Convert to Pixel Art
                </button>
              {/if}
            </div>
          </div>
        {/if}
      </div>
    {:else if $selectedProject}
      <div class="flex-1 overflow-y-auto p-4 space-y-4">
        <!-- Project Name -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            Name
          </label>
          <p class="mt-1 font-medium">{$selectedProject.name}</p>
        </div>

        <!-- Project Path -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            Path
          </label>
          <p class="mt-1 text-sm break-all">{$selectedProject.path}</p>
        </div>

        <!-- Status -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            Status
          </label>
          <p class="mt-1">
            <span
              class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium {$selectedProject.status === 'active' ? 'bg-green-500/20 text-green-400' : 'bg-yellow-500/20 text-yellow-400'}"
            >
              {$selectedProject.status}
            </span>
          </p>
        </div>

        <!-- Created At -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            Created
          </label>
          <p class="mt-1 text-sm">{formatDate($selectedProject.created_at)}</p>
        </div>

        <!-- Updated At -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            Last Modified
          </label>
          <p class="mt-1 text-sm">{formatDate($selectedProject.updated_at)}</p>
        </div>

        <!-- Project ID -->
        <div>
          <label class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider">
            ID
          </label>
          <p class="mt-1 text-xs font-mono break-all">{$selectedProject.id}</p>
        </div>
      </div>
    {:else}
      <!-- Empty state -->
      <div class="flex-1 flex items-center justify-center p-4">
        <div class="text-center text-[var(--color-text-muted)]">
          <svg class="w-12 h-12 mx-auto mb-2 opacity-50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <p class="text-sm">Select a project or asset</p>
        </div>
      </div>
    {/if}
  {/if}
</aside>
