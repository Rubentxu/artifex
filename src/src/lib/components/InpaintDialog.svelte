<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import MaskCanvas from './MaskCanvas.svelte';
  import type { InpaintRequest } from '$lib/types/asset';
  import { writeFile, BaseDirectory } from '@tauri-apps/plugin-fs';
  import { tempDir } from '@tauri-apps/api/path';
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
      const maskBase64 = maskCanvasRef.getMaskBase64();
      const base64Data = maskBase64.replace(/^data:image\/png;base64,/, '');
      const maskBytes = Uint8Array.from(atob(base64Data), c => c.charCodeAt(0));

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
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Inpaint Image"
  width="4xl"
  bordered
  {error}
>
  <div class="space-y-4">
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
        Prompt <span class="text-[var(--color-destructive)]">*</span>
      </label>
      <textarea
        id="inpaint-prompt"
        bind:value={prompt}
        placeholder="Describe what to generate in the masked area..."
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-surface)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none resize-none"
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
        <div class="animate-spin w-6 h-6 border-2 border-[var(--color-neon)] border-t-transparent rounded-full"></div>
        <span class="ml-2 text-sm">Processing...</span>
      </div>
    {/if}
  </div>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleInpaint} disabled={loading || !prompt.trim()}>
      {loading ? 'Processing...' : 'Inpaint'}
    </Button>
  {/snippet}
</Dialog>
