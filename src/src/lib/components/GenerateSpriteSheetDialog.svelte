<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateSpriteSheetRequest, OutputFormat, AssetResponse } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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
        dedup_threshold: dedupThreshold / 100,
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
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Generate Sprite Sheet"
  width="lg"
  bordered
  {error}
>
  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-4">
    <!-- Video Source -->
    <div>
      <label for="ss-source" class="block text-sm font-medium mb-1">
        Video Source
      </label>
      <select
        id="ss-source"
        bind:value={selectedAssetId}
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
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
        class="w-full h-2 rounded-sm appearance-none cursor-pointer bg-[var(--color-surface)] accent-[var(--color-accent)]"
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
        class="w-full h-2 rounded-sm appearance-none cursor-pointer bg-[var(--color-surface)] accent-[var(--color-accent)]"
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
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
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
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
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
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
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
  </form>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleSubmit} disabled={loading || !selectedAssetId}>
      {loading ? 'Generating...' : 'Generate'}
    </Button>
  {/snippet}
</Dialog>
