<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import MaskCanvas from './MaskCanvas.svelte';
  import type { InpaintRequest } from '$lib/types/asset';
  import { writeFile, BaseDirectory } from '@tauri-apps/plugin-fs';
  import { tempDir } from '@tauri-apps/api/path';

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
  let strength = $state(0.8);
  let guidanceScale = $state(7.5);
  let steps = $state(30);
  let maskCanvasRef: MaskCanvas;

  async function handleInpaint() {
    if (!prompt.trim()) {
      error = 'Prompt cannot be empty';
      return;
    }

    error = null;
    loading = true;

    try {
      // Get mask from canvas
      const maskBase64 = maskCanvasRef.getMaskBase64();

      // Convert base64 to bytes
      const base64Data = maskBase64.replace(/^data:image\/png;base64,/, '');
      const maskBytes = Uint8Array.from(atob(base64Data), c => c.charCodeAt(0));

      // Write mask to temp file
      const tempPath = await tempDir();
      const maskFileName = `mask_${Date.now()}.png`;
      const maskPath = `${tempPath}/${maskFileName}`;
      await writeFile(maskPath, maskBytes);

      const request: InpaintRequest = {
        project_id: projectId,
        asset_id: assetId,
        mask_path: maskPath,
        prompt: prompt.trim(),
        negative_prompt: negativePrompt.trim() || undefined,
        strength,
        guidance_scale: guidanceScale,
        steps,
      };

      await assetStore.inpaintImage(request);
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
    strength = 0.8;
    guidanceScale = 7.5;
    steps = 30;
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
    class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 overflow-y-auto"
    onclick={handleClose}
    role="dialog"
    aria-modal="true"
  >
    <!-- Dialog -->
    <div
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-4xl mx-4 my-8 border border-[var(--color-surface)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)]">
        <h2 class="text-lg font-semibold">Inpaint Image</h2>
        <button
          onclick={handleClose}
          class="p-1 rounded hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text-muted)]"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-4 space-y-4 max-h-[70vh] overflow-y-auto">
        {#if error}
          <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
            {error}
          </div>
        {/if}

        <!-- Mask Canvas -->
        <div class="flex justify-center">
          <MaskCanvas
            bind:this={maskCanvasRef}
            imageUrl={imageUrl}
            width={Math.min(imageWidth, 512)}
            height={Math.min(imageHeight, 512)}
            brushSize={20}
          />
        </div>

        <!-- Prompt -->
        <div>
          <label class="block text-sm font-medium mb-1" for="inpaint-prompt">
            Prompt <span class="text-red-400">*</span>
          </label>
          <textarea
            id="inpaint-prompt"
            bind:value={prompt}
            placeholder="Describe what to generate in the masked area..."
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none resize-none"
            rows="2"
          ></textarea>
        </div>

        <!-- Negative Prompt -->
        <div>
          <label class="block text-sm font-medium mb-1" for="inpaint-neg-prompt">
            Negative Prompt
          </label>
          <input
            id="inpaint-neg-prompt"
            type="text"
            bind:value={negativePrompt}
            placeholder="Things to avoid (e.g., blurry, distorted)"
            class="w-full px-3 py-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
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
              <label class="block text-sm mb-1" for="inpaint-strength">
                Strength: {strength.toFixed(2)}
              </label>
              <input
                id="inpaint-strength"
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
              <label class="block text-sm mb-1" for="inpaint-guidance">
                Guidance Scale: {guidanceScale.toFixed(1)}
              </label>
              <input
                id="inpaint-guidance"
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
              <label class="block text-sm mb-1" for="inpaint-steps">
                Inference Steps: {steps}
              </label>
              <input
                id="inpaint-steps"
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
            <div class="animate-spin w-6 h-6 border-2 border-[var(--color-accent)] border-t-transparent rounded-full"></div>
            <span class="ml-2 text-sm">Processing...</span>
          </div>
        {/if}
      </div>

      <!-- Actions -->
      <div class="flex justify-end gap-2 p-4 border-t border-[var(--color-surface)]">
        <button
          onclick={handleClose}
          class="px-4 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors"
          disabled={loading}
        >
          Cancel
        </button>
        <button
          onclick={handleInpaint}
          class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50"
          disabled={loading || !prompt.trim()}
        >
          {loading ? 'Processing...' : 'Inpaint'}
        </button>
      </div>
    </div>
  </div>
{/if}
