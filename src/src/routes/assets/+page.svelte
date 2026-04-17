<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { assetStore, filteredAssets } from '$lib/stores/asset';
  import { selectedProject } from '$lib/stores/project';
  import AssetCard from '$lib/components/AssetCard.svelte';
  import GenerateImageDialog from '$lib/components/GenerateImageDialog.svelte';
  import GenerateAudioDialog from '$lib/components/GenerateAudioDialog.svelte';
  import GenerateTileDialog from '$lib/components/GenerateTileDialog.svelte';
  import GenerateSpriteSheetDialog from '$lib/components/GenerateSpriteSheetDialog.svelte';
  import RemoveBackgroundDialog from '$lib/components/RemoveBackgroundDialog.svelte';
  import ConvertPixelArtDialog from '$lib/components/ConvertPixelArtDialog.svelte';
  import JobHistoryPanel from '$lib/components/JobHistoryPanel.svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import type { AssetKind, AssetResponse } from '$lib/types/asset';

  interface JobCompletedPayload {
    job_id: string;
    asset_ids: string[];
  }

  let showGenerateImageDialog = $state(false);
  let showGenerateAudioDialog = $state(false);
  let showGenerateTileDialog = $state(false);
  let showGenerateSpriteSheetDialog = $state(false);
  let showRemoveBackgroundDialog = $state(false);
  let showConvertPixelArtDialog = $state(false);
  let selectedAssetIdForAction = $state<string | null>(null);
  let importError = $state<string | null>(null);
  let unlistenJobCompleted: (() => void) | null = null;

  const filterKinds: (AssetKind | 'All')[] = ['All', 'Image', 'Sprite', 'Tileset', 'Material', 'Audio', 'Voice', 'Video', 'Other'];

  // Derived list of video assets for sprite sheet generation
  let videoAssets: AssetResponse[] = $derived($assetStore.assets.filter(a => a.kind === 'Video'));

  onMount(async () => {
    if ($selectedProject) {
      assetStore.loadAssets($selectedProject.id);
    }

    // Listen for job-completed events to refresh asset list
    try {
      const { listen } = await import('@tauri-apps/api/event');
      unlistenJobCompleted = await listen<JobCompletedPayload>('job-completed', (event) => {
        console.log('Job completed:', event.payload);
        // Refresh asset list when a job completes
        if ($selectedProject) {
          assetStore.loadAssets($selectedProject.id);
        }
      });
    } catch (e) {
      console.warn('Failed to listen for job-completed event:', e);
    }
  });

  onDestroy(() => {
    if (unlistenJobCompleted) {
      unlistenJobCompleted();
    }
  });

  $effect(() => {
    if ($selectedProject) {
      assetStore.loadAssets($selectedProject.id);
    }
  });

  function handleFilterClick(kind: AssetKind | 'All') {
    assetStore.setFilterKind(kind === 'All' ? null : kind);
  }

  function handleAssetClick(assetId: string) {
    assetStore.selectAsset(assetId === $assetStore.selectedId ? null : assetId);
  }

  async function handleImport() {
    if (!$selectedProject) return;

    importError = null;
    try {
      const selected = await open({
        directory: false,
        multiple: false,
        title: 'Select asset file to import',
      });
      if (selected) {
        const filePath = selected as string;
        const fileName = filePath.split(/[/\\]/).pop() || 'Imported Asset';
        const ext = fileName.split('.').pop()?.toLowerCase() || '';
        let kind: AssetKind = 'Other';
        if (['png', 'jpg', 'jpeg', 'gif', 'webp', 'bmp'].includes(ext)) {
          kind = 'Image';
        } else if (['wav', 'mp3', 'ogg', 'flac', 'aac'].includes(ext)) {
          kind = 'Audio';
        } else if (['mp4', 'avi', 'mov', 'mkv'].includes(ext)) {
          kind = 'Video';
        }
        await assetStore.importAsset($selectedProject.id, filePath, fileName, kind);
      }
    } catch (e) {
      importError = e instanceof Error ? e.message : String(e);
    }
  }

  function handleRemoveBackground(assetId: string) {
    selectedAssetIdForAction = assetId;
    showRemoveBackgroundDialog = true;
  }

  function handleConvertPixelArt(assetId: string) {
    selectedAssetIdForAction = assetId;
    showConvertPixelArtDialog = true;
  }

  function handleGenerateSpriteSheet() {
    showGenerateSpriteSheetDialog = true;
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <header class="flex items-center justify-between px-6 py-4 border-b border-[var(--color-surface)]">
    <h1 class="text-2xl font-bold">Assets</h1>
    <div class="flex items-center gap-2">
      <button
        onclick={() => (showGenerateImageDialog = true)}
        class="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 rounded-lg transition-colors font-medium"
        disabled={!$selectedProject}
        title={$selectedProject ? '' : 'Select a project first'}
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        Generate Image
      </button>
      <button
        onclick={() => (showGenerateAudioDialog = true)}
        class="flex items-center gap-2 px-4 py-2 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg transition-colors font-medium"
        disabled={!$selectedProject}
        title={$selectedProject ? '' : 'Select a project first'}
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
        </svg>
        Generate Audio
      </button>
      <button
        onclick={() => (showGenerateTileDialog = true)}
        class="flex items-center gap-2 px-4 py-2 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg transition-colors font-medium"
        disabled={!$selectedProject}
        title={$selectedProject ? '' : 'Select a project first'}
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
        </svg>
        Generate Tile
      </button>
      <button
        onclick={handleGenerateSpriteSheet}
        class="flex items-center gap-2 px-4 py-2 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg transition-colors font-medium"
        disabled={!$selectedProject || videoAssets.length === 0}
        title={!$selectedProject ? 'Select a project first' : videoAssets.length === 0 ? 'No video assets available' : ''}
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M14.828 14.828a4 4 0 01-5.656 0M9 10h.01M15 10h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
        </svg>
        Generate Sprite Sheet
      </button>
      <button
        onclick={handleImport}
        class="flex items-center gap-2 px-4 py-2 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg transition-colors font-medium"
        disabled={!$selectedProject}
        title={$selectedProject ? '' : 'Select a project first'}
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
        </svg>
        Import
      </button>
    </div>
  </header>

  <!-- Filter bar -->
  <div class="flex items-center gap-2 px-6 py-3 border-b border-[var(--color-surface)] overflow-x-auto">
    {#each filterKinds as kind}
      <button
        onclick={() => handleFilterClick(kind)}
        class="shrink-0 px-3 py-1.5 rounded-lg text-sm font-medium transition-colors
          {($assetStore.filterKind === (kind === 'All' ? null : kind))
            ? 'bg-[var(--color-accent)] text-white'
            : 'bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 text-[var(--color-text)]'}"
      >
        {kind}
      </button>
    {/each}
  </div>

  <!-- Selected Asset Actions -->
  {#if $selectedProject && $assetStore.selectedId}
    {@const selectedAssetData = $filteredAssets.find(a => a.id === $assetStore.selectedId)}
    {#if selectedAssetData && (selectedAssetData.kind === 'Image' || selectedAssetData.kind === 'Sprite' || selectedAssetData.kind === 'Tileset' || selectedAssetData.kind === 'Material')}
      <div class="flex items-center gap-2 px-6 py-2 bg-[var(--color-accent)]/10 border-b border-[var(--color-accent)]/20">
        <span class="text-sm text-[var(--color-accent)] font-medium">Selected:</span>
        <span class="text-sm truncate max-w-xs">{selectedAssetData.name}</span>
        <div class="flex items-center gap-2 ml-auto">
          <button
            onclick={() => handleRemoveBackground($assetStore.selectedId!)}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors text-sm font-medium"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
            </svg>
            Remove Background
          </button>
          <button
            onclick={() => handleConvertPixelArt($assetStore.selectedId!)}
            class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors text-sm font-medium"
          >
            <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
            </svg>
            Convert to Pixel Art
          </button>
        </div>
      </div>
    {/if}
  {/if}

  <!-- Error display -->
  {#if importError}
    <div class="mx-6 mt-3 p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
      {importError}
    </div>
  {/if}

  <!-- Asset Grid -->
  <main class="flex-1 overflow-y-auto p-6">
    {#if !$selectedProject}
      <div class="flex flex-col items-center justify-center h-full text-center">
        <div class="w-24 h-24 rounded-full bg-[var(--color-panel)] flex items-center justify-center mb-4">
          <svg class="w-12 h-12 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </div>
        <h2 class="text-xl font-semibold mb-2">No project selected</h2>
        <p class="text-[var(--color-text-muted)]">Select a project to view its assets</p>
      </div>
    {:else if $filteredAssets.length > 0}
      <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4">
        {#each $filteredAssets as asset (asset.id)}
          <AssetCard
            {asset}
            selected={$assetStore.selectedId === asset.id}
            onclick={() => handleAssetClick(asset.id)}
          />
        {/each}
      </div>
    {:else}
      <div class="flex flex-col items-center justify-center h-full text-center">
        <div class="w-24 h-24 rounded-full bg-[var(--color-panel)] flex items-center justify-center mb-4">
          <svg class="w-12 h-12 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
          </svg>
        </div>
        <h2 class="text-xl font-semibold mb-2">No assets yet</h2>
        <p class="text-[var(--color-text-muted)] mb-4">Import files or generate images to get started</p>
        <div class="flex gap-2">
          <button
            onclick={handleImport}
            class="flex items-center gap-2 px-4 py-2 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg transition-colors font-medium"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
            </svg>
            Import
          </button>
          <button
            onclick={() => (showGenerateImageDialog = true)}
            class="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 rounded-lg transition-colors font-medium"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
            </svg>
            Generate Image
          </button>
          <button
            onclick={() => (showGenerateAudioDialog = true)}
            class="flex items-center gap-2 px-4 py-2 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg transition-colors font-medium"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
            </svg>
            Generate Audio
          </button>
          <button
            onclick={() => (showGenerateTileDialog = true)}
            class="flex items-center gap-2 px-4 py-2 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg transition-colors font-medium"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
            </svg>
            Generate Tile
          </button>
        </div>
      </div>
    {/if}
  </main>

  <!-- Recent Jobs -->
  {#if $selectedProject}
    <JobHistoryPanel projectId={$selectedProject.id} />
  {/if}
</div>

<!-- Generate Image Dialog -->
{#if showGenerateImageDialog && $selectedProject}
  <GenerateImageDialog
    open={showGenerateImageDialog}
    projectId={$selectedProject.id}
    onclose={() => (showGenerateImageDialog = false)}
  />
{/if}

<!-- Generate Audio Dialog -->
{#if showGenerateAudioDialog && $selectedProject}
  <GenerateAudioDialog
    open={showGenerateAudioDialog}
    projectId={$selectedProject.id}
    onclose={() => (showGenerateAudioDialog = false)}
  />
{/if}

<!-- Generate Tile Dialog -->
{#if showGenerateTileDialog && $selectedProject}
  <GenerateTileDialog
    open={showGenerateTileDialog}
    projectId={$selectedProject.id}
    onclose={() => (showGenerateTileDialog = false)}
  />
{/if}

<!-- Generate Sprite Sheet Dialog -->
{#if showGenerateSpriteSheetDialog && $selectedProject}
  <GenerateSpriteSheetDialog
    open={showGenerateSpriteSheetDialog}
    projectId={$selectedProject.id}
    videoAssets={videoAssets}
    onclose={() => (showGenerateSpriteSheetDialog = false)}
  />
{/if}

<!-- Remove Background Dialog -->
{#if showRemoveBackgroundDialog && $selectedProject && selectedAssetIdForAction}
  <RemoveBackgroundDialog
    open={showRemoveBackgroundDialog}
    projectId={$selectedProject.id}
    assetId={selectedAssetIdForAction}
    onclose={() => {
      showRemoveBackgroundDialog = false;
      selectedAssetIdForAction = null;
    }}
  />
{/if}

<!-- Convert Pixel Art Dialog -->
{#if showConvertPixelArtDialog && $selectedProject && selectedAssetIdForAction}
  <ConvertPixelArtDialog
    open={showConvertPixelArtDialog}
    projectId={$selectedProject.id}
    assetId={selectedAssetIdForAction}
    onclose={() => {
      showConvertPixelArtDialog = false;
      selectedAssetIdForAction = null;
    }}
  />
{/if}
