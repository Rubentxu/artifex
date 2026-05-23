<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { OutpaintRequest, OutpaintDirection } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  interface Props {
    open: boolean;
    projectId: string;
    assetId: string;
    imageUrl: string;
    imageWidth: number;
    imageHeight: number;
    onclose: () => void;
  }

  let { open, projectId, assetId, imageUrl, imageWidth, imageHeight, onclose }: Props = $props();

  let loading = $state(false);
  let error = $state<string | null>(null);
  let prompt = $state('');
  let negativePrompt = $state('');
  let direction = $state<OutpaintDirection>('right');
  let extendPixels = $state(256);
  let strength = $state(0.8);
  let guidanceScale = $state(7.5);
  let steps = $state(30);

  let previewWidth = $derived(direction === 'left' || direction === 'right' ? imageWidth + extendPixels : imageWidth);
  let previewHeight = $derived(direction === 'top' || direction === 'bottom' ? imageHeight + extendPixels : imageHeight);

  const directions: { value: OutpaintDirection; label: string; icon: string }[] = [
    { value: 'right', label: 'Right', icon: '→' },
    { value: 'left', label: 'Left', icon: '←' },
    { value: 'top', label: 'Top', icon: '↑' },
    { value: 'bottom', label: 'Bottom', icon: '↓' },
  ];

  async function handleOutpaint() {
    if (!prompt.trim()) {
      error = 'Prompt cannot be empty';
      return;
    }

    error = null;
    loading = true;

    try {
      const request: OutpaintRequest = {
        project_id: projectId,
        asset_id: assetId,
        direction,
        extend_pixels: extendPixels,
        prompt: prompt.trim(),
        negative_prompt: negativePrompt.trim() || undefined,
        strength,
        guidance_scale: guidanceScale,
        steps,
      };

      await assetStore.outpaintImage(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    error = null;
    loading = false;
    prompt = '';
    negativePrompt = '';
    direction = 'right';
    extendPixels = 256;
    strength = 0.8;
    guidanceScale = 7.5;
    steps = 30;
    onclose();
  }
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Outpaint Image"
  width="2xl"
  bordered
  {error}
>
  <div class="space-y-4">
    <!-- Direction Selector -->
    <div>
      <label class="block text-sm font-medium mb-2">Direction</label>
      <div class="grid grid-cols-4 gap-2">
        {#each directions as dir}
          <button
            onclick={() => direction = dir.value}
            class="flex flex-col items-center justify-center p-3 rounded-sm border transition-colors"
            class:bg-[var(--color-neon)]={direction === dir.value}
            class:border-[var(--color-neon)]={direction === dir.value}
            class:text-white={direction === dir.value}
            class:bg-[var(--color-surface)]={direction !== dir.value}
            class:border-[var(--color-surface)]={direction !== dir.value}
          >
            <span class="text-2xl">{dir.icon}</span>
            <span class="text-sm mt-1">{dir.label}</span>
          </button>
        {/each}
      </div>
    </div>

    <!-- Extension Size -->
    <div>
      <label class="block text-sm font-medium mb-1" for="outpaint-extension">
        Extension: {extendPixels}px
      </label>
      <input
        id="outpaint-extension"
        type="range"
        min="64"
        max="1024"
        step="64"
        bind:value={extendPixels}
        class="w-full"
      />
      <div class="flex justify-between text-xs text-[var(--color-text-muted)]">
        <span>64px</span>
        <span>1024px</span>
      </div>
    </div>

    <!-- Preview -->
    <div class="flex justify-center">
      <div
        class="border border-[var(--color-surface)] rounded-sm overflow-hidden bg-[var(--color-surface)]/30"
        style="width: {Math.min(previewWidth, 400)}px; height: {Math.min(previewHeight, 300)}px;"
      >
        <div class="w-full h-full flex items-center justify-center text-[var(--color-text-muted)] text-sm">
          Preview: {previewWidth} × {previewHeight}px
        </div>
      </div>
    </div>

    <!-- Prompt -->
    <div>
      <label class="block text-sm font-medium mb-1" for="outpaint-prompt">
        Prompt <span class="text-[var(--color-destructive)]">*</span>
      </label>
      <textarea
        id="outpaint-prompt"
        bind:value={prompt}
        placeholder="Describe what to generate in the extended area..."
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-surface)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none resize-none"
        rows="2"
      ></textarea>
    </div>

    <!-- Negative Prompt -->
    <div>
      <label class="block text-sm font-medium mb-1" for="outpaint-neg-prompt">
        Negative Prompt
      </label>
      <input
        id="outpaint-neg-prompt"
        type="text"
        bind:value={negativePrompt}
        placeholder="Things to avoid (e.g., blurry, distorted)"
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-surface)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none"
      />
    </div>

    <!-- Advanced Parameters -->
    <details class="group">
      <summary class="cursor-pointer text-sm font-medium text-[var(--color-text-muted)] hover:text-[var(--color-text)]">
        Advanced Parameters
      </summary>
      <div class="mt-3 space-y-3">
        <!-- Strength -->
        <div>
          <label class="block text-sm mb-1" for="outpaint-strength">
            Strength: {strength.toFixed(2)}
          </label>
          <input
            id="outpaint-strength"
            type="range"
            min="0"
            max="1"
            step="0.05"
            bind:value={strength}
            class="w-full"
          />
          <p class="text-xs text-[var(--color-text-muted)]">Higher = more creative changes</p>
        </div>

        <!-- Guidance Scale -->
        <div>
          <label class="block text-sm mb-1" for="outpaint-guidance">
            Guidance Scale: {guidanceScale.toFixed(1)}
          </label>
          <input
            id="outpaint-guidance"
            type="range"
            min="1"
            max="20"
            step="0.5"
            bind:value={guidanceScale}
            class="w-full"
          />
        </div>

        <!-- Steps -->
        <div>
          <label class="block text-sm mb-1" for="outpaint-steps">
            Inference Steps: {steps}
          </label>
          <input
            id="outpaint-steps"
            type="range"
            min="1"
            max="100"
            step="1"
            bind:value={steps}
            class="w-full"
          />
        </div>
      </div>
    </details>

    {#if loading}
      <div class="flex items-center justify-center py-4">
        <div class="animate-spin w-6 h-6 border-2 border-[var(--color-neon)] border-t-transparent rounded-full"></div>
        <span class="ml-2 text-sm">Processing...</span>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleOutpaint} disabled={loading || !prompt.trim()}>
      {loading ? 'Processing...' : 'Outpaint'}
    </Button>
  {/snippet}
</Dialog>
