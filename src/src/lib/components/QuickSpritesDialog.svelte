<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { QuickSpritesRequest, QuickSpritesMode, AssetResponse, ImageGenParams } from '$lib/types/asset';
  import { convertFileSrc } from '@tauri-apps/api/core';

  interface Props {
    open: boolean;
    projectId: string;
    availableAssets?: AssetResponse[]; // Filtered to Image/Sprite for FromImage mode
    onclose: () => void;
  }

  let { open, projectId, availableAssets = [], onclose }: Props = $props();

  let mode = $state<QuickSpritesMode>('FromImage');
  let selectedAssetId = $state<string | null>(null);
  let motionPrompt = $state('');
  let negativePrompt = $state('');

  // FromPrompt mode options
  let imagePrompt = $state('');
  let imageNegativePrompt = $state('');
  let imageWidth = $state(512);
  let imageHeight = $state(512);
  let imageSteps = $state(20);

  // Sprite options
  let fps = $state(10);
  let dedupThreshold = $state(0.03);
  let atlasMaxSize = $state(4096);
  let padding = $state(1);
  let animationName = $state('idle');

  // Video options
  let videoDuration = $state(4);

  let loading = $state(false);
  let error = $state<string | null>(null);

  const sizeOptions = [64, 128, 256, 512, 1024, 2048, 4096];
  const fpsOptions = [5, 8, 10, 12, 15, 20, 24, 30];
  const durationOptions = [2, 3, 4, 5, 6, 7, 8];

  // Filter available assets for FromImage mode
  let selectableAssets = $derived(
    availableAssets.filter(a => a.kind === 'Image' || a.kind === 'Sprite')
  );

  async function handleGenerate() {
    error = null;

    if (!motionPrompt.trim()) {
      error = 'Motion prompt is required';
      return;
    }

    if (mode === 'FromImage' && !selectedAssetId) {
      error = 'Select an asset for From Image mode';
      return;
    }

    if (mode === 'FromPrompt' && !imagePrompt.trim()) {
      error = 'Image prompt is required for From Prompt mode';
      return;
    }

    loading = true;
    try {
      const request: QuickSpritesRequest = {
        project_id: projectId,
        mode,
        source_image_asset_id: mode === 'FromImage' ? selectedAssetId! : undefined,
        motion_prompt: motionPrompt.trim(),
        negative_prompt: negativePrompt.trim() || undefined,
        image_gen_params: mode === 'FromPrompt' ? {
          prompt: imagePrompt.trim(),
          negative_prompt: imageNegativePrompt.trim() || undefined,
          width: imageWidth,
          height: imageHeight,
          steps: imageSteps,
        } : undefined,
        options: {
          fps,
          dedup_threshold: dedupThreshold,
          atlas_max_size: atlasMaxSize,
          padding,
          animation_name: animationName,
          output_format: 'Both',
          video_duration_secs: videoDuration,
        },
      };
      await assetStore.generateQuickSprites(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    mode = 'FromImage';
    selectedAssetId = null;
    motionPrompt = '';
    negativePrompt = '';
    imagePrompt = '';
    imageNegativePrompt = '';
    imageWidth = 512;
    imageHeight = 512;
    imageSteps = 20;
    fps = 10;
    dedupThreshold = 0.03;
    atlasMaxSize = 4096;
    padding = 1;
    animationName = 'idle';
    videoDuration = 4;
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
        <h2 class="text-lg font-semibold">Quick Sprites</h2>
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
              onclick={() => mode = 'FromImage'}
              class="flex-1 px-4 py-2 rounded-lg border transition-colors text-sm font-medium
                {mode === 'FromImage'
                  ? 'bg-[var(--color-accent)] border-[var(--color-accent)] text-white'
                  : 'bg-[var(--color-surface)] border-[var(--color-surface)] hover:border-[var(--color-accent)]/50'}"
            >
              From Image
            </button>
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
          </div>
        </div>

        {#if mode === 'FromImage'}
          <!-- From Image Mode -->
          <div>
            <label class="block text-sm font-medium mb-2">
              Select Image Asset ({selectedAssetId ? '1 selected' : 'none selected'})
            </label>
            {#if selectableAssets.length > 0}
              <div class="grid grid-cols-4 gap-2 max-h-48 overflow-y-auto p-1">
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
                No Image/Sprite assets available. Import some images first.
              </p>
            {/if}
          </div>
        {:else}
          <!-- From Prompt Mode -->
          <div class="space-y-4">
            <!-- Image Prompt -->
            <div>
              <label for="qs-image-prompt" class="block text-sm font-medium mb-1">
                Image Prompt <span class="text-red-400">*</span>
              </label>
              <textarea
                id="qs-image-prompt"
                bind:value={imagePrompt}
                placeholder="A character sprite sheet for a platformer game"
                rows="2"
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
              ></textarea>
            </div>

            <!-- Image Negative Prompt -->
            <div>
              <label for="qs-image-neg-prompt" class="block text-sm font-medium mb-1">
                Negative Prompt
              </label>
              <textarea
                id="qs-image-neg-prompt"
                bind:value={imageNegativePrompt}
                placeholder="blurry, low quality, artifacts"
                rows="1"
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
              ></textarea>
            </div>

            <!-- Image Size -->
            <div class="grid grid-cols-2 gap-4">
              <div>
                <label for="qs-img-width" class="block text-sm font-medium mb-1">Width</label>
                <select
                  id="qs-img-width"
                  bind:value={imageWidth}
                  class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
                >
                  {#each sizeOptions as size}
                    <option value={size}>{size}</option>
                  {/each}
                </select>
              </div>
              <div>
                <label for="qs-img-height" class="block text-sm font-medium mb-1">Height</label>
                <select
                  id="qs-img-height"
                  bind:value={imageHeight}
                  class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
                >
                  {#each sizeOptions as size}
                    <option value={size}>{size}</option>
                  {/each}
                </select>
              </div>
            </div>

            <!-- Steps -->
            <div>
              <label for="qs-steps" class="block text-sm font-medium mb-1">Steps: {imageSteps}</label>
              <input
                id="qs-steps"
                type="range"
                bind:value={imageSteps}
                min="10"
                max="50"
                step="5"
                class="w-full"
              />
            </div>
          </div>
        {/if}

        <!-- Motion Prompt (shared) -->
        <div>
          <label for="qs-motion-prompt" class="block text-sm font-medium mb-1">
            Motion Prompt <span class="text-red-400">*</span>
          </label>
          <textarea
            id="qs-motion-prompt"
            bind:value={motionPrompt}
            placeholder="Character walking animation, 4-frame loop"
            rows="2"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
          ></textarea>
        </div>

        <!-- Negative Prompt (shared) -->
        <div>
          <label for="qs-neg-prompt" class="block text-sm font-medium mb-1">
            Negative Prompt
          </label>
          <textarea
            id="qs-neg-prompt"
            bind:value={negativePrompt}
            placeholder="blur, jitter, artifacts"
            rows="1"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
          ></textarea>
        </div>

        <!-- Advanced Options (collapsible) -->
        <details class="group">
          <summary class="flex items-center cursor-pointer text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)]">
            <svg class="w-4 h-4 mr-1 transition-transform group-open:rotate-90" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
            Advanced Options
          </summary>
          <div class="mt-3 space-y-3 pl-5">
            <!-- Video Duration -->
            <div>
              <label for="qs-duration" class="block text-sm font-medium mb-1">
                Video Duration: {videoDuration}s
              </label>
              <select
                id="qs-duration"
                bind:value={videoDuration}
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
              >
                {#each durationOptions as dur}
                  <option value={dur}>{dur}s</option>
                {/each}
              </select>
            </div>

            <!-- FPS -->
            <div>
              <label for="qs-fps" class="block text-sm font-medium mb-1">FPS: {fps}</label>
              <select
                id="qs-fps"
                bind:value={fps}
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
              >
                {#each fpsOptions as f}
                  <option value={f}>{f} fps</option>
                {/each}
              </select>
            </div>

            <!-- Dedup Threshold -->
            <div>
              <label for="qs-dedup" class="block text-sm font-medium mb-1">
                Dedup Threshold: {dedupThreshold.toFixed(3)}
              </label>
              <input
                id="qs-dedup"
                type="range"
                bind:value={dedupThreshold}
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

            <!-- Atlas Max Size -->
            <div>
              <label for="qs-atlas" class="block text-sm font-medium mb-1">Atlas Max Size: {atlasMaxSize}</label>
              <select
                id="qs-atlas"
                bind:value={atlasMaxSize}
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
              >
                {#each [1024, 2048, 4096, 8192] as size}
                  <option value={size}>{size}x{size}</option>
                {/each}
              </select>
            </div>

            <!-- Padding -->
            <div>
              <label for="qs-padding" class="block text-sm font-medium mb-1">
                Padding: {padding}px
              </label>
              <input
                id="qs-padding"
                type="range"
                bind:value={padding}
                min="0"
                max="8"
                step="1"
                class="w-full"
              />
            </div>

            <!-- Animation Name -->
            <div>
              <label for="qs-anim-name" class="block text-sm font-medium mb-1">Animation Name</label>
              <input
                id="qs-anim-name"
                type="text"
                bind:value={animationName}
                placeholder="idle"
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
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
          disabled={loading || (mode === 'FromImage' && !selectedAssetId) || (mode === 'FromPrompt' && !imagePrompt.trim()) || !motionPrompt.trim()}
        >
          {loading ? 'Generating...' : 'Generate'}
        </button>
      </div>
    </div>
  </div>
{/if}