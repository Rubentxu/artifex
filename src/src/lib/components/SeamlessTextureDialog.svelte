<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { SeamlessTextureRequest, SeamlessMode, AssetResponse } from '$lib/types/asset';
  import { convertFileSrc } from '@tauri-apps/api/core';

  interface Props {
    open: boolean;
    projectId: string;
    availableAssets?: AssetResponse[]; // Filtered to Image/Sprite/Tileset for FromAsset mode
    onclose: () => void;
  }

  let { open, projectId, availableAssets = [], onclose }: Props = $props();

  let mode = $state<SeamlessMode>('FromAsset');
  let prompt = $state('');
  let negativePrompt = $state('');
  let width = $state(512);
  let height = $state(512);
  let selectedAssetId = $state<string | null>(null);
  let seamThreshold = $state(0.05);
  let paddingPixels = $state(16);
  let blendFraction = $state(0.5);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const sizeOptions = [64, 128, 256, 512, 1024, 2048, 4096];

  // Filter available assets for FromAsset mode
  let selectableAssets = $derived(
    availableAssets.filter(a => a.kind === 'Image' || a.kind === 'Sprite' || a.kind === 'Tileset' || a.kind === 'Material')
  );

  async function handleGenerate() {
    error = null;

    if (mode === 'FromPrompt' && !prompt.trim()) {
      error = 'Prompt is required for From Prompt mode';
      return;
    }

    if (mode === 'FromAsset' && !selectedAssetId) {
      error = 'Select an asset for From Asset mode';
      return;
    }

    loading = true;
    try {
      const request: SeamlessTextureRequest = {
        project_id: projectId,
        mode,
        prompt: mode === 'FromPrompt' ? prompt.trim() : undefined,
        negative_prompt: mode === 'FromPrompt' && negativePrompt.trim() ? negativePrompt.trim() : undefined,
        width: mode === 'FromPrompt' ? width : undefined,
        height: mode === 'FromPrompt' ? height : undefined,
        asset_id: mode === 'FromAsset' ? selectedAssetId! : undefined,
        seam_threshold: seamThreshold,
        padding_pixels: paddingPixels,
        blend_fraction: blendFraction,
      };
      await assetStore.generateSeamlessTexture(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    mode = 'FromAsset';
    prompt = '';
    negativePrompt = '';
    width = 512;
    height = 512;
    selectedAssetId = null;
    seamThreshold = 0.05;
    paddingPixels = 16;
    blendFraction = 0.5;
    error = null;
    loading = false;
    onclose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }

  function selectAsset(assetId: string) {
    selectedAssetId = selectedAssetId === assetId ? null : assetId;
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- Backdrop -->
  <div
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
    onclick={handleClose}
    role="dialog"
    aria-modal="true"
  >
    <!-- Dialog -->
    <div
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-2xl mx-4 border border-[var(--color-surface)] max-h-[90vh] overflow-hidden flex flex-col"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)] shrink-0">
        <h2 class="text-lg font-semibold">Seamless Texture</h2>
        <button
          onclick={handleClose}
          class="p-1 rounded hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text-muted)]"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-4 space-y-4 overflow-y-auto flex-1">
        {#if error}
          <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
            {error}
          </div>
        {/if}

        <!-- Mode Toggle -->
        <div>
          <label class="block text-sm font-medium mb-2">Mode</label>
          <div class="flex gap-2">
            <button
              type="button"
              onclick={() => mode = 'FromPrompt'}
              class="flex-1 px-4 py-2 rounded-lg border transition-colors text-sm font-medium
                {mode === 'FromPrompt'
                  ? 'bg-[var(--color-accent)] border-[var(--color-accent)] text-white'
                  : 'bg-[var(--color-surface)] border-[var(--color-surface)] hover:border-[var(--color-accent)]/50'}"
            >
              From Prompt
            </button>
            <button
              type="button"
              onclick={() => mode = 'FromAsset'}
              class="flex-1 px-4 py-2 rounded-lg border transition-colors text-sm font-medium
                {mode === 'FromAsset'
                  ? 'bg-[var(--color-accent)] border-[var(--color-accent)] text-white'
                  : 'bg-[var(--color-surface)] border-[var(--color-surface)] hover:border-[var(--color-accent)]/50'}"
            >
              From Asset
            </button>
          </div>
        </div>

        {#if mode === 'FromPrompt'}
          <!-- From Prompt Panel -->
          <div class="space-y-4">
            <!-- Prompt -->
            <div>
              <label for="st-prompt" class="block text-sm font-medium mb-1">
                Prompt <span class="text-red-400">*</span>
              </label>
              <textarea
                id="st-prompt"
                bind:value={prompt}
                placeholder="A seamless stone texture for dungeon floors"
                rows="3"
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
              ></textarea>
            </div>

            <!-- Negative Prompt -->
            <div>
              <label for="st-neg-prompt" class="block text-sm font-medium mb-1">
                Negative Prompt
              </label>
              <textarea
                id="st-neg-prompt"
                bind:value={negativePrompt}
                placeholder="blur, noise, artifacts"
                rows="2"
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
              ></textarea>
            </div>

            <!-- Size -->
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label for="st-width" class="block text-sm font-medium mb-1">Width</label>
                <select
                  id="st-width"
                  bind:value={width}
                  class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
                >
                  {#each sizeOptions as size}
                    <option value={size}>{size}</option>
                  {/each}
                </select>
              </div>
              <div>
                <label for="st-height" class="block text-sm font-medium mb-1">Height</label>
                <select
                  id="st-height"
                  bind:value={height}
                  class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
                >
                  {#each sizeOptions as size}
                    <option value={size}>{size}</option>
                  {/each}
                </select>
              </div>
            </div>
          </div>
        {:else}
          <!-- From Asset Panel -->
          <div>
            <label class="block text-sm font-medium mb-2">
              Select Asset ({selectedAssetId ? '1 selected' : 'none selected'})
            </label>
            {#if selectableAssets.length > 0}
              <div class="grid grid-cols-4 gap-2 max-h-64 overflow-y-auto p-1">
                {#each selectableAssets as asset (asset.id)}
                  {@const selected = selectedAssetId === asset.id}
                  <button
                    type="button"
                    onclick={() => selectAsset(asset.id)}
                    class="relative p-2 rounded-lg border-2 transition-colors text-left
                      {selected
                        ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10'
                        : 'border-[var(--color-surface)] hover:border-[var(--color-accent)]/50'}"
                  >
                    {#if asset.file_path}
                      <img
                        src={convertFileSrc(asset.file_path)}
                        alt={asset.name}
                        class="w-full aspect-square object-cover rounded"
                      />
                    {:else}
                      <div class="w-full aspect-square bg-[var(--color-surface)] rounded flex items-center justify-center text-xs text-[var(--color-text-muted)]">
                        {asset.name.slice(0, 6)}
                      </div>
                    {/if}
                    <div class="mt-1 text-xs truncate">{asset.name}</div>
                    {#if selected}
                      <div class="absolute top-1 right-1 w-5 h-5 bg-[var(--color-accent)] rounded-full flex items-center justify-center">
                        <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                        </svg>
                      </div>
                    {/if}
                  </button>
                {/each}
              </div>
            {:else}
              <p class="text-sm text-[var(--color-text-muted)] text-center py-8">
                No Image/Sprite/Tileset/Material assets available. Import some images first.
              </p>
            {/if}
          </div>
        {/if}

        <!-- Advanced Options (collapsible) -->
        <details class="group">
          <summary class="flex items-center cursor-pointer text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)]">
            <svg class="w-4 h-4 mr-1 transition-transform group-open:rotate-90" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
            Advanced Options
          </summary>
          <div class="mt-3 space-y-3 pl-5">
            <!-- Seam Threshold -->
            <div>
              <label for="st-threshold" class="block text-sm font-medium mb-1">
                Seam Threshold: {seamThreshold.toFixed(3)} (MAE)
              </label>
              <input
                id="st-threshold"
                type="range"
                bind:value={seamThreshold}
                min="0.01"
                max="0.20"
                step="0.01"
                class="w-full"
              />
              <div class="flex justify-between text-xs text-[var(--color-text-muted)]">
                <span>Strict</span>
                <span>Lenient</span>
              </div>
            </div>

            <!-- Padding Pixels -->
            <div>
              <label for="st-padding" class="block text-sm font-medium mb-1">
                Padding Pixels: {paddingPixels}
              </label>
              <input
                id="st-padding"
                type="range"
                bind:value={paddingPixels}
                min="4"
                max="64"
                step="4"
                class="w-full"
              />
            </div>

            <!-- Blend Fraction -->
            <div>
              <label for="st-blend" class="block text-sm font-medium mb-1">
                Blend Fraction: {blendFraction.toFixed(2)}
              </label>
              <input
                id="st-blend"
                type="range"
                bind:value={blendFraction}
                min="0.1"
                max="0.9"
                step="0.1"
                class="w-full"
              />
            </div>
          </div>
        </details>
      </div>

      <!-- Footer -->
      <div class="flex justify-end gap-2 p-4 border-t border-[var(--color-surface)] shrink-0">
        <button
          type="button"
          onclick={handleClose}
          class="px-4 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors"
          disabled={loading}
        >
          Cancel
        </button>
        <button
          type="button"
          onclick={handleGenerate}
          class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50"
          disabled={loading || (mode === 'FromAsset' && !selectedAssetId) || (mode === 'FromPrompt' && !prompt.trim())}
        >
          {loading ? 'Generating...' : 'Generate'}
        </button>
      </div>
    </div>
  </div>
{/if}