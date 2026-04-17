<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { AssetResponse, CreateAnimationRequest } from '$lib/types/asset';

  interface Props {
    open: boolean;
    projectId: string;
    availableFrames: AssetResponse[];
    onclose: () => void;
  }

  let { open, projectId, availableFrames, onclose }: Props = $props();

  let name = $state('New Animation');
  let selectedFrameIds = $state<string[]>([]);
  let fps = $state(12);
  let loading = $state(false);
  let error = $state<string | null>(null);

  async function handleCreate() {
    error = null;

    if (!name.trim()) {
      error = 'Animation name is required';
      return;
    }

    if (selectedFrameIds.length === 0) {
      error = 'Select at least one frame';
      return;
    }

    loading = true;
    try {
      const request: CreateAnimationRequest = {
        project_id: projectId,
        name: name.trim(),
        frame_asset_ids: selectedFrameIds,
        default_fps: fps,
      };
      await assetStore.createAnimation(request);
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    name = 'New Animation';
    selectedFrameIds = [];
    fps = 12;
    error = null;
    onclose();
  }

  function toggleFrame(frameId: string) {
    if (selectedFrameIds.includes(frameId)) {
      selectedFrameIds = selectedFrameIds.filter(id => id !== frameId);
    } else {
      selectedFrameIds = [...selectedFrameIds, frameId];
    }
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
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-2xl mx-4 border border-[var(--color-surface)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between px-6 py-4 border-b border-[var(--color-surface)]">
        <h2 class="text-xl font-bold">Create Animation</h2>
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
      <div class="p-6 space-y-4">
        {#if error}
          <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
            {error}
          </div>
        {/if}

        <!-- Name -->
        <div>
          <label class="block text-sm font-medium mb-1.5" for="anim-name">Animation Name</label>
          <input
            id="anim-name"
            type="text"
            bind:value={name}
            class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-lg border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
            placeholder="My Animation"
          />
        </div>

        <!-- FPS -->
        <div>
          <label class="block text-sm font-medium mb-1.5" for="anim-fps">Default FPS</label>
          <input
            id="anim-fps"
            type="number"
            bind:value={fps}
            min="1"
            max="120"
            class="w-24 px-3 py-2 bg-[var(--color-surface)] rounded-lg border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none"
          />
          <span class="ml-2 text-sm text-[var(--color-text-muted)]">
            ({Math.round(1000 / fps)}ms per frame)
          </span>
        </div>

        <!-- Frame selection -->
        <div>
          <label class="block text-sm font-medium mb-1.5">
            Select Frames ({selectedFrameIds.length} selected)
          </label>
          <div class="grid grid-cols-4 gap-2 max-h-64 overflow-y-auto p-1">
            {#each availableFrames as frame (frame.id)}
              {@const selected = selectedFrameIds.includes(frame.id)}
              <button
                onclick={() => toggleFrame(frame.id)}
                class="relative p-2 rounded-lg border-2 transition-colors text-left
                  {selected
                    ? 'border-[var(--color-accent)] bg-[var(--color-accent)]/10'
                    : 'border-[var(--color-surface)] hover:border-[var(--color-accent)]/50'}"
              >
                {#if frame.file_path}
                  <img
                    src={frame.file_path}
                    alt={frame.name}
                    class="w-full aspect-square object-cover rounded"
                  />
                {:else}
                  <div class="w-full aspect-square bg-[var(--color-surface)] rounded flex items-center justify-center text-xs text-[var(--color-text-muted)]">
                    {frame.name.slice(0, 6)}
                  </div>
                {/if}
                <div class="mt-1 text-xs truncate">{frame.name}</div>
                {#if selected}
                  <div class="absolute top-1 right-1 w-5 h-5 bg-[var(--color-accent)] rounded-full flex items-center justify-center">
                    <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="3" d="M5 13l4 4L19 7" />
                    </svg>
                  </div>
                {/if}
              </button>
            {/each}
          </div>
          {#if availableFrames.length === 0}
            <p class="text-sm text-[var(--color-text-muted)] text-center py-8">
              No image/sprite assets available. Import some images first.
            </p>
          {/if}
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
          onclick={handleCreate}
          disabled={loading || selectedFrameIds.length === 0}
          class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50 disabled:cursor-not-allowed"
        >
          {#if loading}
            Creating...
          {:else}
            Create Animation
          {/if}
        </button>
      </div>
    </div>
  </div>
{/if}
