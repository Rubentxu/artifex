<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { AssetResponse, PackAtlasRequest, PackAtlasOptions, AtlasSortMode } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  interface Props {
    open: boolean;
    projectId: string;
    availableAssets: AssetResponse[];
    onclose: () => void;
  }

  let { open, projectId, availableAssets, onclose }: Props = $props();

  let atlasName = $state('');
  let selectedAssetIds = $state<string[]>([]);
  let maxSize = $state<512 | 1024 | 2048 | 4096>(2048);
  let padding = $state(1);
  let allowRotation = $state(false);
  let sortMode = $state<AtlasSortMode>('None');
  let loading = $state(false);
  let error = $state<string | null>(null);

  const maxSizeOptions: { value: 512 | 1024 | 2048 | 4096; label: string }[] = [
    { value: 512, label: '512 x 512' },
    { value: 1024, label: '1024 x 1024' },
    { value: 2048, label: '2048 x 2048' },
    { value: 4096, label: '4096 x 4096' },
  ];

  const sortModeOptions: { value: AtlasSortMode; label: string }[] = [
    { value: 'None', label: 'None (Default)' },
    { value: 'Area', label: 'Area (Largest First)' },
    { value: 'MaxSide', label: 'Max Side' },
    { value: 'Width', label: 'Width' },
    { value: 'Height', label: 'Height' },
  ];

  async function handlePack() {
    error = null;

    if (!atlasName.trim()) {
      error = 'Atlas name is required';
      return;
    }

    if (selectedAssetIds.length < 2) {
      error = 'Select at least 2 assets';
      return;
    }

    if (padding < 0 || padding > 16) {
      error = 'Padding must be between 0 and 16';
      return;
    }

    loading = true;
    try {
      const request: PackAtlasRequest = {
        project_id: projectId,
        atlas_name: atlasName.trim(),
        source_asset_ids: selectedAssetIds,
        options: {
          max_size: maxSize,
          padding,
          allow_rotation: allowRotation,
          sort_mode: sortMode,
        },
      };
      await assetStore.packAtlas(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    atlasName = '';
    selectedAssetIds = [];
    maxSize = 2048;
    padding = 1;
    allowRotation = false;
    sortMode = 'None';
    error = null;
    onclose();
  }

  function toggleAsset(assetId: string) {
    if (selectedAssetIds.includes(assetId)) {
      selectedAssetIds = selectedAssetIds.filter(id => id !== assetId);
    } else {
      selectedAssetIds = [...selectedAssetIds, assetId];
    }
  }
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Pack Texture Atlas"
  width="2xl"
  bordered
  scrollBody
  {error}
>
  <div class="space-y-4">
    <!-- Atlas Name -->
    <div>
      <label class="block text-sm font-medium mb-1.5" for="atlas-name">Atlas Name</label>
      <input
        id="atlas-name"
        type="text"
        bind:value={atlasName}
        class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-sm border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none"
        placeholder="my_atlas"
      />
    </div>

    <!-- Options Row -->
    <div class="grid grid-cols-2 gap-4">
      <!-- Max Size -->
      <div>
        <label for="atlas-max-size" class="block text-sm font-medium mb-1">
          Max Size
        </label>
        <select
          id="atlas-max-size"
          bind:value={maxSize}
          class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-sm border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none"
        >
          {#each maxSizeOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </div>

      <!-- Padding -->
      <div>
        <label for="atlas-padding" class="block text-sm font-medium mb-1">
          Padding (px)
        </label>
        <input
          id="atlas-padding"
          type="number"
          bind:value={padding}
          min="0"
          max="16"
          class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-sm border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none"
        />
      </div>
    </div>

    <!-- Options Row 2 -->
    <div class="grid grid-cols-2 gap-4">
      <!-- Allow Rotation -->
      <div class="flex items-center gap-2">
        <input
          id="atlas-rotation"
          type="checkbox"
          bind:checked={allowRotation}
          class="w-4 h-4 rounded border-[var(--color-surface)] text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
        />
        <label for="atlas-rotation" class="text-sm font-medium">Allow Rotation</label>
      </div>

      <!-- Sort Mode -->
      <div>
        <label for="atlas-sort-mode" class="block text-sm font-medium mb-1">
          Sort Mode
        </label>
        <select
          id="atlas-sort-mode"
          bind:value={sortMode}
          class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-sm border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none"
        >
          {#each sortModeOptions as opt}
            <option value={opt.value}>{opt.label}</option>
          {/each}
        </select>
      </div>
    </div>

    <!-- Asset Selection -->
    <div>
      <label class="block text-sm font-medium mb-1.5">
        Select Assets ({selectedAssetIds.length} selected, minimum 2)
      </label>
      {#if selectedAssetIds.length < 2}
        <p class="text-xs text-[var(--color-text-muted)] mb-2">Select at least 2 assets to pack</p>
      {/if}
      <div class="grid grid-cols-4 gap-2 max-h-64 overflow-y-auto p-1">
        {#each availableAssets as asset (asset.id)}
          {@const selected = selectedAssetIds.includes(asset.id)}
          <button
            onclick={() => toggleAsset(asset.id)}
            class="relative p-2 rounded-sm border-2 transition-colors text-left
              {selected
                ? 'border-[var(--color-neon)] bg-[var(--color-neon)]/10'
                : 'border-[var(--color-surface)] hover:border-[var(--color-neon)]/50'}"
          >
            {#if asset.file_path}
              <img
                src={asset.file_path}
                alt={asset.name}
                class="w-full aspect-square object-cover rounded"
              />
            {:else}
              <div class="w-full aspect-square bg-[var(--color-surface)] rounded flex items-center justify-center text-xs text-[var(--color-text-muted)]">
                {asset.name.slice(0, 6)}
              </div>
            {/if}
            <div class="mt-1 text-xs truncate">{asset.name}</div>
            <div class="text-xs text-[var(--color-text-muted)]">{asset.kind}</div>
            {#if selected}
              <div class="absolute top-1 right-1 w-5 h-5 bg-[var(--color-neon)] rounded-full flex items-center justify-center">
                <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                </svg>
              </div>
            {/if}
          </button>
        {/each}
      </div>
      {#if availableAssets.length === 0}
        <p class="text-sm text-[var(--color-text-muted)] text-center py-8">
          No Image/Sprite/Tileset assets available. Import some assets first.
        </p>
      {/if}
    </div>
  </div>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose}>
      Cancel
    </Button>
    <Button onclick={handlePack} disabled={loading || selectedAssetIds.length < 2 || !atlasName.trim()}>
      {#if loading}
        Packing...
      {:else}
        Pack Atlas
      {/if}
    </Button>
  {/snippet}
</Dialog>
