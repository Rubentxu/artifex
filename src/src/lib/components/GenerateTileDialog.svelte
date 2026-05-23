<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateTileRequest } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Generate Tile"
  width="lg"
  bordered
  {error}
>
  <form onsubmit={(e) => { e.preventDefault(); handleGenerate(); }} class="space-y-4">
    <!-- Prompt -->
    <div>
      <label for="tile-prompt" class="block text-sm font-medium mb-1">
        Prompt <span class="text-[var(--color-destructive)]">*</span>
      </label>
      <textarea
        id="tile-prompt"
        bind:value={prompt}
        placeholder="A seamless stone texture for dungeon floors"
        rows="3"
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none"
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
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
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
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
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
  </form>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleGenerate} disabled={loading}>
      {loading ? 'Generating...' : 'Generate'}
    </Button>
  {/snippet}
</Dialog>
