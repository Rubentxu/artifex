<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateTileRequest } from '$lib/types/asset';

  interface Props {
    open: boolean;
    projectId: string;
    onclose: () => void;
  }

  let { open, projectId, onclose }: Props = $props();

  let prompt = $state('');
  let width = $state(256);
  let height = $state(256);
  let biome = $state('generic');
  let seamless = $state(true);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const sizeOptions = [
    { value: 256, label: '256x256' },
    { value: 512, label: '512x512' },
  ];

  const biomeOptions = [
    { value: 'generic', label: 'Generic' },
    { value: 'forest', label: 'Forest' },
    { value: 'dungeon', label: 'Dungeon' },
    { value: 'sky', label: 'Sky' },
    { value: 'desert', label: 'Desert' },
    { value: 'snow', label: 'Snow' },
    { value: 'cave', label: 'Cave' },
  ];

  async function handleGenerate() {
    error = null;

    if (!prompt.trim()) {
      error = 'Prompt is required';
      return;
    }

    loading = true;
    try {
      const request: GenerateTileRequest = {
        project_id: projectId,
        prompt: prompt.trim(),
        width,
        height,
        biome,
        seamless,
      };
      await assetStore.generateTile(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    prompt = '';
    width = 256;
    height = 256;
    biome = 'generic';
    seamless = true;
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
        <h2 class="text-lg font-semibold">Generate Tile</h2>
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

        <!-- Prompt -->
        <div>
          <label for="tile-prompt" class="block text-sm font-medium mb-1">
            Prompt <span class="text-red-400">*</span>
          </label>
          <textarea
            id="tile-prompt"
            bind:value={prompt}
            placeholder="A seamless stone texture for dungeon floors"
            rows="3"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
            required
          ></textarea>
        </div>

        <!-- Size -->
        <div>
          <label for="tile-size" class="block text-sm font-medium mb-1">
            Size
          </label>
          <select
            id="tile-size"
            bind:value={width}
            onchange={() => height = width}
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
          >
            {#each sizeOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <!-- Biome -->
        <div>
          <label for="tile-biome" class="block text-sm font-medium mb-1">
            Biome
          </label>
          <select
            id="tile-biome"
            bind:value={biome}
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
          >
            {#each biomeOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <!-- Seamless -->
        <div class="flex items-center gap-3">
          <input
            id="tile-seamless"
            type="checkbox"
            bind:checked={seamless}
            class="w-4 h-4 rounded border-[var(--color-surface)] text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
          />
          <label for="tile-seamless" class="text-sm font-medium">
            Seamless (tileable texture)
          </label>
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
            disabled={loading}
          >
            {loading ? 'Generating...' : 'Generate'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}