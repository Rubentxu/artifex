<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { SliceSpriteSheetRequest, SliceMode, AssetResponse, GridSliceParams, AutoDetectSliceParams, SortOrder } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  interface Props {
    open: boolean;
    projectId: string;
    imageAssets: AssetResponse[];
    onclose: () => void;
  }

  let { open, projectId, imageAssets, onclose }: Props = $props();

  let selectedAssetId = $state(imageAssets.length > 0 ? imageAssets[0].id : '');
  let mode = $state<SliceMode>('Grid');
  let loading = $state(false);
  let error = $state<string | null>(null);

  let rows = $state(4);
  let cols = $state(4);
  let margin = $state(0);

  let minArea = $state(100);
  let sortOrder = $state<SortOrder>('TopToBottom');

  const gridRowsOptions = [
    { value: 2, label: '2 rows' },
    { value: 3, label: '3 rows' },
    { value: 4, label: '4 rows' },
    { value: 5, label: '5 rows' },
    { value: 6, label: '6 rows' },
    { value: 8, label: '8 rows' },
    { value: 10, label: '10 rows' },
  ];

  const gridColsOptions = [
    { value: 2, label: '2 cols' },
    { value: 3, label: '3 cols' },
    { value: 4, label: '4 cols' },
    { value: 5, label: '5 cols' },
    { value: 6, label: '6 cols' },
    { value: 8, label: '8 cols' },
    { value: 10, label: '10 cols' },
  ];

  const sortOrderOptions: { value: SortOrder; label: string }[] = [
    { value: 'TopToBottom', label: 'Top to Bottom' },
    { value: 'LeftToRight', label: 'Left to Right' },
  ];

  async function handleSubmit() {
    error = null;

    if (!selectedAssetId) {
      error = 'Please select an image asset';
      return;
    }

    if (rows < 1 || rows > 20) {
      error = 'Rows must be between 1 and 20';
      return;
    }

    if (cols < 1 || cols > 20) {
      error = 'Columns must be between 1 and 20';
      return;
    }

    if (margin < 0 || margin > 50) {
      error = 'Margin must be between 0 and 50';
      return;
    }

    if (minArea < 1) {
      error = 'Minimum area must be at least 1';
      return;
    }

    loading = true;
    try {
      const request: SliceSpriteSheetRequest = {
        asset_id: selectedAssetId,
        project_id: projectId,
        mode,
        grid_params: {
          rows,
          cols,
          margin,
        },
        auto_detect_params: {
          min_area: minArea,
          sort_order: sortOrder,
        },
      };
      await assetStore.sliceSpriteSheet(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    selectedAssetId = imageAssets.length > 0 ? imageAssets[0].id : '';
    mode = 'Grid';
    rows = 4;
    cols = 4;
    margin = 0;
    minArea = 100;
    sortOrder = 'TopToBottom';
    error = null;
    loading = false;
    onclose();
  }
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Slice Sprite Sheet"
  width="lg"
  bordered
  scrollBody
  {error}
>
  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-4">
    <!-- Image Source -->
    <div>
      <label for="ss-source" class="block text-sm font-medium mb-1">
        Image Source
      </label>
      <select
        id="ss-source"
        bind:value={selectedAssetId}
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
      >
        {#if imageAssets.length === 0}
          <option value="">No image assets available</option>
        {:else}
          {#each imageAssets as asset}
            <option value={asset.id}>{asset.name}</option>
          {/each}
        {/if}
      </select>
    </div>

    <!-- Mode Toggle -->
    <div>
      <label class="block text-sm font-medium mb-2">
        Slice Mode
      </label>
      <div class="flex gap-4">
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name="slice-mode"
            value="Grid"
            bind:group={mode}
            class="w-4 h-4 text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
          />
          <span class="text-sm">Grid</span>
        </label>
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="radio"
            name="slice-mode"
            value="AutoDetect"
            bind:group={mode}
            class="w-4 h-4 text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
          />
          <span class="text-sm">Auto-detect</span>
        </label>
      </div>
    </div>

    {#if mode === 'Grid'}
      <!-- Grid Mode Parameters -->
      <div class="space-y-4 pl-2 border-l-2 border-[var(--color-neon)]/30">
        <!-- Rows -->
        <div>
          <label for="ss-rows" class="block text-sm font-medium mb-1">
            Rows: {rows}
          </label>
          <select
            id="ss-rows"
            bind:value={rows}
            class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
          >
            {#each gridRowsOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <!-- Columns -->
        <div>
          <label for="ss-cols" class="block text-sm font-medium mb-1">
            Columns: {cols}
          </label>
          <select
            id="ss-cols"
            bind:value={cols}
            class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
          >
            {#each gridColsOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <!-- Margin -->
        <div>
          <label for="ss-margin" class="block text-sm font-medium mb-1">
            Margin (px): {margin}
          </label>
          <input
            id="ss-margin"
            type="number"
            bind:value={margin}
            min="0"
            max="50"
            class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
          />
        </div>
      </div>
    {:else}
      <!-- Auto-detect Mode Parameters -->
      <div class="space-y-4 pl-2 border-l-2 border-[var(--color-neon)]/30">
        <!-- Minimum Area -->
        <div>
          <label for="ss-min-area" class="block text-sm font-medium mb-1">
            Minimum Sprite Area (px): {minArea}
          </label>
          <input
            id="ss-min-area"
            type="number"
            bind:value={minArea}
            min="1"
            class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
          />
          <p class="text-xs text-[var(--color-text-muted)] mt-1">
            Sprites smaller than this will be ignored
          </p>
        </div>

        <!-- Sort Order -->
        <div>
          <label class="block text-sm font-medium mb-2">
            Sort Order
          </label>
          <div class="flex gap-4">
            {#each sortOrderOptions as opt}
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="sort-order"
                  value={opt.value}
                  bind:group={sortOrder}
                  class="w-4 h-4 text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                />
                <span class="text-sm">{opt.label}</span>
              </label>
            {/each}
          </div>
        </div>
      </div>
    {/if}
  </form>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleSubmit} disabled={loading || !selectedAssetId}>
      {loading ? 'Slicing...' : 'Slice'}
    </Button>
  {/snippet}
</Dialog>
