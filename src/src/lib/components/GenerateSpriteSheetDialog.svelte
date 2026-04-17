<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateSpriteSheetRequest, OutputFormat, AssetResponse } from '$lib/types/asset';

  interface Props {
    open: boolean;
    projectId: string;
    videoAssets: AssetResponse[];
    onclose: () => void;
  }

  let { open, projectId, videoAssets, onclose }: Props = $props();

  let selectedAssetId = $state(videoAssets.length > 0 ? videoAssets[0].id : '');
  let fps = $state(10);
  let dedupThreshold = $state(3);
  let atlasMaxSize = $state(4096);
  let padding = $state(1);
  let animationName = $state('idle');
  let outputFormat = $state<OutputFormat>('Both');
  let loading = $state(false);
  let error = $state<string | null>(null);

  const fpsOptions = [
    { value: 1, label: '1 fps' },
    { value: 5, label: '5 fps' },
    { value: 10, label: '10 fps' },
    { value: 15, label: '15 fps' },
    { value: 24, label: '24 fps' },
    { value: 30, label: '30 fps' },
  ];

  const atlasSizeOptions = [
    { value: 1024, label: '1024 x 1024' },
    { value: 2048, label: '2048 x 2048' },
    { value: 4096, label: '4096 x 4096' },
  ];

  const outputFormatOptions: { value: OutputFormat; label: string }[] = [
    { value: 'Both', label: 'Both (JSON + Aseprite)' },
    { value: 'Json', label: 'JSON only' },
    { value: 'Aseprite', label: 'Aseprite only' },
  ];

  async function handleSubmit() {
    error = null;

    if (!selectedAssetId) {
      error = 'Please select a video asset';
      return;
    }

    if (!animationName.trim()) {
      error = 'Animation name is required';
      return;
    }

    if (fps < 1 || fps > 30) {
      error = 'FPS must be between 1 and 30';
      return;
    }

    if (dedupThreshold < 0 || dedupThreshold > 10) {
      error = 'Dedup threshold must be between 0 and 10';
      return;
    }

    if (padding < 0 || padding > 4) {
      error = 'Padding must be between 0 and 4';
      return;
    }

    loading = true;
    try {
      const request: GenerateSpriteSheetRequest = {
        asset_id: selectedAssetId,
        project_id: projectId,
        fps,
        dedup_threshold: dedupThreshold / 100, // Convert percentage to 0-1 range
        atlas_max_size: atlasMaxSize,
        padding,
        animation_name: animationName.trim(),
        output_format: outputFormat,
      };
      await assetStore.generateSpriteSheet(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    // Reset form values
    selectedAssetId = videoAssets.length > 0 ? videoAssets[0].id : '';
    fps = 10;
    dedupThreshold = 3;
    atlasMaxSize = 4096;
    padding = 1;
    animationName = 'idle';
    outputFormat = 'Both';
    error = null;
    loading = false;
    onclose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
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
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-lg mx-4 border border-[var(--color-surface)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)]">
        <h2 class="text-lg font-semibold">Generate Sprite Sheet</h2>
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
      <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="p-4 space-y-4">
        {#if error}
          <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
            {error}
          </div>
        {/if}

        <!-- Video Source -->
        <div>
          <label for="ss-source" class="block text-sm font-medium mb-1">
            Video Source
          </label>
          <select
            id="ss-source"
            bind:value={selectedAssetId}
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
          >
            {#if videoAssets.length === 0}
              <option value="">No video assets available</option>
            {:else}
              {#each videoAssets as asset}
                <option value={asset.id}>{asset.name}</option>
              {/each}
            {/if}
          </select>
        </div>

        <!-- FPS -->
        <div>
          <label for="ss-fps" class="block text-sm font-medium mb-1">
            FPS: {fps}
          </label>
          <input
            id="ss-fps"
            type="range"
            bind:value={fps}
            min="1"
            max="30"
            step="1"
            class="w-full h-2 rounded-lg appearance-none cursor-pointer bg-[var(--color-surface)] accent-[var(--color-accent)]"
          />
          <div class="flex justify-between text-xs text-[var(--color-text-muted)] mt-1">
            <span>1</span>
            <span>30</span>
          </div>
        </div>

        <!-- Dedup Threshold -->
        <div>
          <label for="ss-dedup" class="block text-sm font-medium mb-1">
            Dedup Threshold: {dedupThreshold}%
          </label>
          <input
            id="ss-dedup"
            type="range"
            bind:value={dedupThreshold}
            min="0"
            max="10"
            step="0.5"
            class="w-full h-2 rounded-lg appearance-none cursor-pointer bg-[var(--color-surface)] accent-[var(--color-accent)]"
          />
          <div class="flex justify-between text-xs text-[var(--color-text-muted)] mt-1">
            <span>0% (keep all)</span>
            <span>10%</span>
          </div>
        </div>

        <!-- Atlas Max Size -->
        <div>
          <label for="ss-atlas-size" class="block text-sm font-medium mb-1">
            Atlas Max Size
          </label>
          <select
            id="ss-atlas-size"
            bind:value={atlasMaxSize}
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
          >
            {#each atlasSizeOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <!-- Padding -->
        <div>
          <label for="ss-padding" class="block text-sm font-medium mb-1">
            Frame Padding (px)
          </label>
          <input
            id="ss-padding"
            type="number"
            bind:value={padding}
            min="0"
            max="4"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
          />
        </div>

        <!-- Animation Name -->
        <div>
          <label for="ss-anim-name" class="block text-sm font-medium mb-1">
            Animation Name
          </label>
          <input
            id="ss-anim-name"
            type="text"
            bind:value={animationName}
            placeholder="e.g., idle, walk, jump"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
          />
        </div>

        <!-- Output Format -->
        <div>
          <label class="block text-sm font-medium mb-2">
            Output Format
          </label>
          <div class="flex gap-4">
            {#each outputFormatOptions as opt}
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="output-format"
                  value={opt.value}
                  bind:group={outputFormat}
                  class="w-4 h-4 text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                />
                <span class="text-sm">{opt.label}</span>
              </label>
            {/each}
          </div>
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
            disabled={loading || !selectedAssetId}
          >
            {loading ? 'Generating...' : 'Generate'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
