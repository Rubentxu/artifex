<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { SliceSpriteSheetRequest, SliceMode, AssetResponse, GridSliceParams, AutoDetectSliceParams, SortOrder } from '$lib/types/asset';

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

  // Grid params
  let rows = $state(4);
  let cols = $state(4);
  let margin = $state(0);

  // Auto-detect params
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
    // Reset form values
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
        <h2 class="text-lg font-semibold">Slice Sprite Sheet</h2>
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

        <!-- Image Source -->
        <div>
          <label for="ss-source" class="block text-sm font-medium mb-1">
            Image Source
          </label>
          <select
            id="ss-source"
            bind:value={selectedAssetId}
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
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
          <div class="space-y-4 pl-2 border-l-2 border-[var(--color-accent)]/30">
            <!-- Rows -->
            <div>
              <label for="ss-rows" class="block text-sm font-medium mb-1">
                Rows: {rows}
              </label>
              <select
                id="ss-rows"
                bind:value={rows}
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
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
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
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
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
              />
            </div>
          </div>
        {:else}
          <!-- Auto-detect Mode Parameters -->
          <div class="space-y-4 pl-2 border-l-2 border-[var(--color-accent)]/30">
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
                class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
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
            {loading ? 'Slicing...' : 'Slice'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
