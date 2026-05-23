<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateImageRequest } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  interface Props {
    open: boolean;
    projectId: string;
    onclose: () => void;
  }

  let { open, projectId, onclose }: Props = $props();

  let prompt = $state('');
  let negativePrompt = $state('');
  let width = $state(512);
  let height = $state(512);
  let steps = $state(20);
  let seed = $state<number | undefined>(undefined);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function handleGenerate() {
    error = null;

    if (!prompt.trim()) {
      error = 'Prompt is required';
      return;
    }

    loading = true;
    try {
      const request: GenerateImageRequest = {
        project_id: projectId,
        prompt: prompt.trim(),
        negative_prompt: negativePrompt.trim() || undefined,
        width,
        height,
        steps,
        seed,
      };
      await assetStore.generateImage(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    prompt = '';
    negativePrompt = '';
    width = 512;
    height = 512;
    steps = 20;
    seed = undefined;
    error = null;
    onclose();
  }
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Generate Image"
  width="lg"
  bordered
  {error}
>
  <form onsubmit={(e) => { e.preventDefault(); handleGenerate(); }} class="space-y-4">
    <!-- Prompt -->
    <div>
      <label for="gen-prompt" class="block text-sm font-medium mb-1">
        Prompt <span class="text-[var(--color-destructive)]">*</span>
      </label>
      <textarea
        id="gen-prompt"
        bind:value={prompt}
        placeholder="A serene landscape with mountains at sunset"
        rows="4"
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none"
        required
      ></textarea>
    </div>

    <!-- Negative Prompt -->
    <div>
      <label for="gen-negative-prompt" class="block text-sm font-medium mb-1">
        Negative Prompt
      </label>
      <textarea
        id="gen-negative-prompt"
        bind:value={negativePrompt}
        placeholder="blurry, low quality, distorted"
        rows="2"
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none"
      ></textarea>
    </div>

    <!-- Dimensions -->
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label for="gen-width" class="block text-sm font-medium mb-1">
          Width
        </label>
        <input
          id="gen-width"
          type="number"
          bind:value={width}
          min="64"
          max="2048"
          step="64"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
      <div>
        <label for="gen-height" class="block text-sm font-medium mb-1">
          Height
        </label>
        <input
          id="gen-height"
          type="number"
          bind:value={height}
          min="64"
          max="2048"
          step="64"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
    </div>

    <!-- Steps and Seed -->
    <div class="grid grid-cols-2 gap-4">
      <div>
        <label for="gen-steps" class="block text-sm font-medium mb-1">
          Steps
        </label>
        <input
          id="gen-steps"
          type="number"
          bind:value={steps}
          min="1"
          max="100"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
      <div>
        <label for="gen-seed" class="block text-sm font-medium mb-1">
          Seed (optional)
        </label>
        <input
          id="gen-seed"
          type="number"
          bind:value={seed}
          placeholder="Random"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
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
