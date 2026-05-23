<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { AssetResponse, CreateAnimationRequest } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Create Animation"
  width="2xl"
  bordered
  scrollBody
  {error}
>
  <div class="space-y-4">
    <!-- Name -->
    <div>
      <label class="block text-sm font-medium mb-1.5" for="anim-name">Animation Name</label>
      <input
        id="anim-name"
        type="text"
        bind:value={name}
        class="w-full px-3 py-2 bg-[var(--color-surface)] rounded-sm border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none"
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
        class="w-24 px-3 py-2 bg-[var(--color-surface)] rounded-sm border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none"
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
            class="relative p-2 rounded-sm border-2 transition-colors text-left
              {selected
                ? 'border-[var(--color-neon)] bg-[var(--color-neon)]/10'
                : 'border-[var(--color-surface)] hover:border-[var(--color-neon)]/50'}"
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
              <div class="absolute top-1 right-1 w-5 h-5 bg-[var(--color-neon)] rounded-full flex items-center justify-center">
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

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose}>
      Cancel
    </Button>
    <Button onclick={handleCreate} disabled={loading || selectedFrameIds.length === 0}>
      {#if loading}
        Creating...
      {:else}
        Create Animation
      {/if}
    </Button>
  {/snippet}
</Dialog>
