<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateImageRequest } from '$lib/types/asset';

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
        <h2 class="text-lg font-semibold">Generate Image</h2>
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
          <label for="gen-prompt" class="block text-sm font-medium mb-1">
            Prompt <span class="text-red-400">*</span>
          </label>
          <textarea
            id="gen-prompt"
            bind:value={prompt}
            placeholder="A serene landscape with mountains at sunset"
            rows="4"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
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
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
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
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
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
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
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
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
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
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
            />
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
            disabled={loading}
          >
            {loading ? 'Generating...' : 'Generate'}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}
