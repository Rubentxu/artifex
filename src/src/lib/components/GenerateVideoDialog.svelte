<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateVideoRequest, AssetResponse } from '$lib/types/asset';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  interface Props {
    open: boolean;
    projectId: string;
    availableAssets?: AssetResponse[];
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

<Dialog
  {open}
  onClose={handleClose}
  title="Generate Video"
  width="2xl"
  bordered
  scrollBody
  {error}
>
  <form onsubmit={(e) => { e.preventDefault(); handleGenerate(); }} class="space-y-4">
    <!-- Source Image Selection -->
    <div>
      <label class="block text-sm font-medium mb-2">
        Source Image <span class="text-[var(--color-destructive)]">*</span>
      </label>
      {#if selectableAssets.length === 0}
        <p class="text-sm text-[var(--color-text-muted)] p-3 bg-[var(--color-surface)] rounded-sm">
          No Image or Sprite assets available. Import an image first.
        </p>
      {:else}
        <div class="grid grid-cols-4 gap-2 max-h-48 overflow-y-auto p-1">
          {#each selectableAssets as asset}
            <button
              type="button"
              onclick={() => selectAsset(asset.id)}
              class="relative rounded-sm overflow-hidden border-2 transition-colors {selectedAssetId === asset.id ? 'border-[var(--color-neon)]' : 'border-transparent hover:border-[var(--color-surface)]'}"
            >
              <img
                src={getImageUrl(asset)}
                alt={asset.name}
                class="w-full h-20 object-cover"
              />
              {#if selectedAssetId === asset.id}
                <div class="absolute inset-0 bg-[var(--color-neon)]/30 flex items-center justify-center">
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
        Prompt <span class="text-[var(--color-destructive)]">*</span>
      </label>
      <textarea
        id="video-prompt"
        bind:value={prompt}
        placeholder="Describe the motion, e.g., 'The car drives through the city streets at sunset'"
        rows="3"
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none"
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
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none"
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
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
      />
    </div>
  </form>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleGenerate} disabled={loading || !selectedAssetId || !prompt.trim()}>
      {loading ? 'Generating...' : 'Generate Video'}
    </Button>
  {/snippet}
</Dialog>
