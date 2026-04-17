<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { CameraAngle, CameraPreset, Render3dRequest } from '$lib/types/asset';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';

  interface Props {
    open: boolean;
    projectId: string;
    onclose: () => void;
  }

  let { open, projectId, onclose }: Props = $props();

  let modelFilePath = $state('');
  let cameraPreset = $state<CameraPreset>('isometric');
  let customAngles = $state<CameraAngle[]>([]);
  let outputWidth = $state(256);
  let outputHeight = $state(256);
  let loading = $state(false);
  let error = $state<string | null>(null);

  const cameraPresetOptions: { value: CameraPreset; label: string }[] = [
    { value: 'isometric', label: 'Isometric (8 directions)' },
    { value: 'topdown', label: 'Top-down (4 directions)' },
    { value: 'custom', label: 'Custom angles' },
  ];

  async function handleBrowse() {
    try {
      const selected = await openDialog({
        directory: false,
        multiple: false,
        title: 'Select 3D model file',
        filters: [
          { name: '3D Models', extensions: ['gltf', 'glb', 'obj'] },
        ],
      });
      if (selected) {
        modelFilePath = selected as string;
      }
    } catch (e) {
      console.error('Failed to open file dialog:', e);
    }
  }

  function addCustomAngle() {
    customAngles = [...customAngles, { yawDegrees: 0, pitchDegrees: 45 }];
  }

  function removeCustomAngle(index: number) {
    customAngles = customAngles.filter((_, i) => i !== index);
  }

  function updateCustomAngle(index: number, field: 'yawDegrees' | 'pitchDegrees', value: number) {
    customAngles = customAngles.map((angle, i) =>
      i === index ? { ...angle, [field]: value } : angle
    );
  }

  async function handleRender() {
    error = null;

    if (!modelFilePath.trim()) {
      error = 'Model file path is required';
      return;
    }

    if (cameraPreset === 'custom' && customAngles.length === 0) {
      error = 'At least one custom angle is required';
      return;
    }

    if (outputWidth < 32 || outputWidth > 4096) {
      error = 'Output width must be between 32 and 4096';
      return;
    }

    if (outputHeight < 32 || outputHeight > 4096) {
      error = 'Output height must be between 32 and 4096';
      return;
    }

    loading = true;
    try {
      const request: Render3dRequest = {
        projectId: projectId,
        modelFilePath: modelFilePath.trim(),
        cameraPreset,
        customAngles: cameraPreset === 'custom' ? customAngles : undefined,
        outputWidth,
        outputHeight,
      };
      await assetStore.render3d(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    modelFilePath = '';
    cameraPreset = 'isometric';
    customAngles = [];
    outputWidth = 256;
    outputHeight = 256;
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
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-lg mx-4 border border-[var(--color-surface)] max-h-[90vh] flex flex-col"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-[var(--color-surface)]">
        <h2 class="text-xl font-bold">Render 3D Model to Sprites</h2>
        <button
          onclick={handleClose}
          class="p-1 rounded-lg hover:bg-[var(--color-surface)] transition-colors"
        >
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      <!-- Content -->
      <div class="p-6 space-y-4 overflow-y-auto flex-1">
        {#if error}
          <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
            {error}
          </div>
        {/if}

        <!-- Model File -->
        <div>
          <label class="block text-sm font-medium mb-1.5" for="model-file">Model File (GLTF/GLB/OBJ)</label>
          <div class="flex gap-2">
            <input
              id="model-file"
              type="text"
              bind:value={modelFilePath}
              class="flex-1 px-3 py-2 bg-[var(--color-surface)] rounded-lg border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
              placeholder="/path/to/model.gltf"
            />
            <button
              onclick={handleBrowse}
              class="px-4 py-2 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg border border-[var(--color-surface)] transition-colors"
            >
              Browse
            </button>
          </div>
        </div>

        <!-- Camera Preset -->
        <div>
          <label for="camera-preset" class="block text-sm font-medium mb-1.5">
            Camera Preset
          </label>
          <select
            id="camera-preset"
            bind:value={cameraPreset}
            class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-lg border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
          >
            {#each cameraPresetOptions as opt}
              <option value={opt.value}>{opt.label}</option>
            {/each}
          </select>
        </div>

        <!-- Custom Angles -->
        {#if cameraPreset === 'custom'}
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label class="text-sm font-medium">Custom Angles</label>
              <button
                onclick={addCustomAngle}
                class="text-sm text-[var(--color-accent)] hover:text-[var(--color-accent)]/80"
              >
                + Add Angle
              </button>
            </div>
            {#if customAngles.length === 0}
              <p class="text-xs text-[var(--color-text-muted)] mb-2">Add at least one camera angle</p>
            {/if}
            <div class="space-y-2 max-h-40 overflow-y-auto">
              {#each customAngles as angle, index}
                <div class="flex items-center gap-2">
                  <div class="flex-1 grid grid-cols-2 gap-2">
                    <div>
                      <label class="text-xs text-[var(--color-text-muted)]">Yaw (°)</label>
                      <input
                        type="number"
                        value={angle.yawDegrees}
                        onchange={(e) => updateCustomAngle(index, 'yawDegrees', parseFloat(e.currentTarget.value) || 0)}
                        class="w-full px-2 py-1 bg-[var(--color-surface)] rounded border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
                      />
                    </div>
                    <div>
                      <label class="text-xs text-[var(--color-text-muted)]">Pitch (°)</label>
                      <input
                        type="number"
                        value={angle.pitchDegrees}
                        onchange={(e) => updateCustomAngle(index, 'pitchDegrees', parseFloat(e.currentTarget.value) || 0)}
                        class="w-full px-2 py-1 bg-[var(--color-surface)] rounded border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
                      />
                    </div>
                  </div>
                  <button
                    onclick={() => removeCustomAngle(index)}
                    class="p-1 text-red-400 hover:text-red-300"
                  >
                    <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- Output Size -->
        <div>
          <label class="block text-sm font-medium mb-1.5">Output Size</label>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label for="output-width" class="text-xs text-[var(--color-text-muted)]">Width (px)</label>
              <input
                id="output-width"
                type="number"
                bind:value={outputWidth}
                min="32"
                max="4096"
                class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-lg border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
              />
            </div>
            <div>
              <label for="output-height" class="text-xs text-[var(--color-text-muted)]">Height (px)</label>
              <input
                id="output-height"
                type="number"
                bind:value={outputHeight}
                min="32"
                max="4096"
                class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-lg border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
              />
            </div>
          </div>
        </div>

        <!-- Info -->
        <div class="text-xs text-[var(--color-text-muted)]">
          {#if cameraPreset === 'isometric'}
            <p>Renders 8 frames from isometric angles (every 45°)</p>
          {:else if cameraPreset === 'topdown'}
            <p>Renders 4 frames from top-down angles (every 90°)</p>
          {:else}
            <p>Renders {customAngles.length} frame{customAngles.length !== 1 ? 's' : ''} from custom angles</p>
          {/if}
          <p class="mt-1">Output will be packed into a sprite atlas.</p>
        </div>
      </div>

      <!-- Footer -->
      <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-[var(--color-surface)]">
        <button
          onclick={handleClose}
          class="px-4 py-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors font-medium"
        >
          Cancel
        </button>
        <button
          onclick={handleRender}
          disabled={loading || !modelFilePath.trim() || (cameraPreset === 'custom' && customAngles.length === 0)}
          class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if loading}
            Rendering...
          {:else}
            Render
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
