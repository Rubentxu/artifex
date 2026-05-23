<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { ConvertPixelArtRequest, PaletteMode, DitheringMode } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  interface Props {
    open: boolean;
    projectId: string;
    assetId: string;
    onclose: () => void;
  }

  let { open, projectId, assetId, onclose }: Props = $props();

  let targetWidth = $state(64);
  let targetHeight = $state(64);
  let palette = $state<PaletteMode>({ type: 'Pico8' });
  let dithering = $state<DitheringMode>('none');
  let outline = $state(false);
  let outlineThreshold = $state(128);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const paletteOptions: { value: PaletteMode; label: string }[] = [
    { value: { type: 'Pico8' }, label: 'Pico-8 (16 colors)' },
    { value: { type: 'GameBoy' }, label: 'GameBoy (4 colors)' },
    { value: { type: 'NES' }, label: 'NES (54 colors)' },
  ];

  const ditheringOptions: { value: DitheringMode; label: string }[] = [
    { value: 'none', label: 'None' },
    { value: 'floyd_steinberg', label: 'Floyd-Steinberg' },
  ];

  async function handleConvert() {
    error = null;

    if (targetWidth < 1 || targetHeight < 1) {
      error = 'Dimensions must be at least 1x1';
      return;
    }

    loading = true;
    try {
      const request: ConvertPixelArtRequest = {
        project_id: projectId,
        asset_id: assetId,
        target_width: targetWidth,
        target_height: targetHeight,
        palette,
        dithering,
        outline,
        outline_threshold: outlineThreshold,
      };
      await assetStore.convertPixelArt(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    targetWidth = 64;
    targetHeight = 64;
    palette = { type: 'Pico8' };
    dithering = 'none';
    outline = false;
    outlineThreshold = 128;
    error = null;
    loading = false;
    onclose();
  }
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Convert to Pixel Art"
  width="lg"
  bordered
  {error}
>
  <form onsubmit={(e) => { e.preventDefault(); handleConvert(); }} class="space-y-4">
    <!-- Dimensions -->
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label for="pa-width" class="block text-sm font-medium mb-1">
          Width (px)
        </label>
        <input
          id="pa-width"
          type="number"
          bind:value={targetWidth}
          min="1"
          max="1024"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
      <div>
        <label for="pa-height" class="block text-sm font-medium mb-1">
          Height (px)
        </label>
        <input
          id="pa-height"
          type="number"
          bind:value={targetHeight}
          min="1"
          max="1024"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
    </div>

    <!-- Palette -->
    <div>
      <label for="pa-palette" class="block text-sm font-medium mb-1">
        Palette
      </label>
      <select
        id="pa-palette"
        bind:value={palette}
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
      >
        {#each paletteOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>

    <!-- Dithering -->
    <div>
      <label for="pa-dithering" class="block text-sm font-medium mb-1">
        Dithering
      </label>
      <select
        id="pa-dithering"
        bind:value={dithering}
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
      >
        {#each ditheringOptions as opt}
          <option value={opt.value}>{opt.label}</option>
        {/each}
      </select>
    </div>

    <!-- Outline -->
    <div class="flex items-center gap-3">
      <input
        id="pa-outline"
        type="checkbox"
        bind:checked={outline}
        class="w-4 h-4 rounded border-[var(--color-surface)] text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
      />
      <label for="pa-outline" class="text-sm font-medium">
        Apply outline (edge detection)
      </label>
    </div>

    {#if outline}
      <div>
        <label for="pa-outline-threshold" class="block text-sm font-medium mb-1">
          Outline Threshold (0-255)
        </label>
        <input
          id="pa-outline-threshold"
          type="number"
          bind:value={outlineThreshold}
          min="0"
          max="255"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
    {/if}
  </form>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleConvert} disabled={loading}>
      {loading ? 'Converting...' : 'Convert'}
    </Button>
  {/snippet}
</Dialog>
