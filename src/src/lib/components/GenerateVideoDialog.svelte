<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateVideoRequest, AssetResponse } from '$lib/types/asset';
  import { convertFileSrc } from '@tauri-apps/api/core';

  interface Props {
    open: boolean;
    projectId: string;
    availableAssets?: AssetResponse[]; // Filtered to Image/Sprite for source selection
    onclose: () => void;
  }

  let { open, projectId, availableAssets = [], onclose }: Props = $props();

  let selectedAssetId = $state<string | null>(null);
  let prompt = $state('');
  let negativePrompt = $state('');
  let durationSecs = $state(4);
  let seed = $state<number | undefined>(undefined);
  let loading = $state(false);
  let error = $state<string | null>(null);

  // Filter available assets for source image selection
  let selectableAssets = $derived(
    availableAssets.filter(a => a.kind === 'Image' || a.kind === 'Sprite')
  );

  async function handleGenerate() {
    error = null;

    if (!selectedAssetId) {
      error = 'Select a source image';
      return;
    }

    if (!prompt.trim()) {
      error = 'Prompt is required';
      return;
    }

    loading = true;
    try {
      const request: GenerateVideoRequest = {
        project_id: projectId,
        source_image_asset_id: selectedAssetId!,
        prompt: prompt.trim(),
        negative_prompt: negativePrompt.trim() || undefined,
        duration_secs: durationSecs,
        seed: seed,
      };
      await assetStore.generateVideo(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    selectedAssetId = null;
    prompt = '';
    negativePrompt = '';
    durationSecs = 4;
    seed = undefined;
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

  function getImageUrl(asset: AssetResponse): string {
    if (asset.file_path) {
      return convertFileSrc(asset.file_path);
    }
    return '';
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
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-2xl mx-4 border border-[var(--color-surface)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)]">
        <h2 class="text-lg font-semibold">Generate Video</h2>
        <button
          onclick={handleClose}
          class="p-1 rounded hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text-muted)]"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Form -->
      <form onsubmit={(e) => { e.preventDefault(); handleGenerate(); }} class="p-4 space-y-4">
        {#if error}
          <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
            {error}
          </div>
        {/if}

        <!-- Source Image Selection -->
        <div>
          <label class="block text-sm font-medium mb-2">
            Source Image <span class="text-red-400">*</span>
          </label>
          {#if selectableAssets.length === 0}
            <p class="text-sm text-[var(--color-text-muted)] p-3 bg-[var(--color-surface)] rounded-lg">
              No Image or Sprite assets available. Import an image first.
            </p>
          {:else}
            <div class="grid grid-cols-4 gap-2 max-h-48 overflow-y-auto p-1">
              {#each selectableAssets as asset}
                <button
                  type="button"
                  onclick={() => selectAsset(asset.id)}
                  class="relative rounded-lg overflow-hidden border-2 transition-colors {selectedAssetId === asset.id ? 'border-[var(--color-accent)]' : 'border-transparent hover:border-[var(--color-surface)]'}"
                >
                  <img
                    src={getImageUrl(asset)}
                    alt={asset.name}
                    class="w-full h-20 object-cover"
                  />
                  {#if selectedAssetId === asset.id}
                    <div class="absolute inset-0 bg-[var(--color-accent)]/30 flex items-center justify-center">
                      <svg class="w-6 h-6 text-white" fill="currentColor" viewBox="0 0 24 24">
                        <path d="M9 16.17L4.83 12l-1.42 1.41L9 19 21 7l-1.41-1.41L9 16.17z"/>
                      </svg>
                    </div>
                  {/if}
                </button>
              {/each}
            </div>
            <p class="text-xs text-[var(--color-text-muted)] mt-1">
              {selectedAssetId ? '1 image selected' : 'Click to select a source image'}
            </p>
          {/if}
        </div>

        <!-- Prompt -->
        <div>
          <label for="video-prompt" class="block text-sm font-medium mb-1">
            Prompt <span class="text-red-400">*</span>
          </label>
          <textarea
            id="video-prompt"
            bind:value={prompt}
            placeholder="Describe the motion, e.g., 'The car drives through the city streets at sunset'"
            rows="3"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
            required
          ></textarea>
        </div>

        <!-- Duration Slider -->
        <div>
          <label for="video-duration" class="block text-sm font-medium mb-1">
            Duration: {durationSecs}s
          </label>
          <input
            id="video-duration"
            type="range"
            bind:value={durationSecs}
            min="2"
            max="8"
            step="1"
            class="w-full"
          />
          <div class="flex justify-between text-xs text-[var(--color-text-muted)]">
            <span>2s</span>
            <span>8s</span>
          </div>
        </div>

        <!-- Negative Prompt -->
        <div>
          <label for="video-negative-prompt" class="block text-sm font-medium mb-1">
            Negative Prompt
          </label>
          <textarea
            id="video-negative-prompt"
            bind:value={negativePrompt}
            placeholder="Things to avoid, e.g., blurry, jittery, distorted"
            rows="2"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
          ></textarea>
        </div>

        <!-- Seed (optional) -->
        <div>
          <label for="video-seed" class="block text-sm font-medium mb-1">
            Seed (optional)
          </label>
          <input
            id="video-seed"
            type="number"
            bind:value={seed}
            placeholder="Random"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
          />
        </div>

        <!-- Actions -->
        <div class="flex justify-end gap-2 pt-2">
          <button
            type="button"
            onclick={handleClose}
            class="px-4 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors"
            disabled={loading}
          >
            Cancel
          </button>
          <button
            type="submit"
            class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50"
            disabled={loading || !selectedAssetId || !prompt.trim()}
          >
            {loading ? 'Generating...' : 'Generate Video'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
