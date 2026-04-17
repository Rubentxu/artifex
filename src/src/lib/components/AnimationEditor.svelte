<script lang="ts">
  import { convertFileSrc } from '@tauri-apps/api/core';
  import CanvasPlayer from './CanvasPlayer.svelte';
  import type { AnimationMetadata, AnimationResponse, AssetResponse, UpdateAnimationRequest } from '$lib/types/asset';
  import { assetStore } from '$lib/stores/asset';

  interface Props {
    animation: AnimationResponse;
    projectId: string;
    allAssets: AssetResponse[];
    onclose: () => void;
  }

  let { animation, projectId, allAssets, onclose }: Props = $props();

  // Parse metadata
  let metadata = $state<AnimationMetadata>(
    animation.metadata ?? {
      name: animation.name,
      frame_asset_ids: [],
      frame_durations_ms: [],
      loop_animation: true,
      total_duration_ms: 0,
    }
  );

  let editingName = $state(false);
  let nameInput = $state(metadata.name);
  let saving = $state(false);
  let error = $state<string | null>(null);

  // Build frame URLs map from available assets
  let frameUrls = $state<Map<string, string>>(new Map());

  // Available image/sprite assets for adding frames
  let availableFrames = $derived(
    allAssets.filter(a => a.kind === 'Image' || a.kind === 'Sprite')
  );

  // Frames with URLs for display
  let framesWithUrls = $derived(
    metadata.frame_asset_ids.map(id => {
      const asset = allAssets.find(a => a.id === id);
      return {
        id,
        url: asset?.file_path ? convertFileSrc(asset.file_path) : '',
        duration_ms: metadata.frame_durations_ms[metadata.frame_asset_ids.indexOf(id)] || 100,
        name: asset?.name || id,
      };
    })
  );

  async function handleSave() {
    saving = true;
    error = null;
    try {
      const request: UpdateAnimationRequest = {
        id: animation.id,
        name: metadata.name,
        frame_asset_ids: metadata.frame_asset_ids,
        frame_durations_ms: metadata.frame_durations_ms,
        loop_animation: metadata.loop_animation,
      };
      await assetStore.updateAnimation(request);
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  function handleNameSave() {
    metadata.name = nameInput;
    editingName = false;
    handleSave();
  }

  function handleDurationChange(frameId: string, newDuration: string) {
    const index = metadata.frame_asset_ids.indexOf(frameId);
    if (index !== -1) {
      const duration = parseInt(newDuration, 10);
      if (!isNaN(duration) && duration > 0) {
        metadata.frame_durations_ms[index] = duration;
        metadata.total_duration_ms = metadata.frame_durations_ms.reduce((a, b) => a + b, 0);
        handleSave();
      }
    }
  }

  function removeFrame(frameId: string) {
    const index = metadata.frame_asset_ids.indexOf(frameId);
    if (index !== -1) {
      metadata.frame_asset_ids.splice(index, 1);
      metadata.frame_durations_ms.splice(index, 1);
      metadata.total_duration_ms = metadata.frame_durations_ms.reduce((a, b) => a + b, 0);
      handleSave();
    }
  }

  function moveFrame(fromIndex: number, toIndex: number) {
    if (toIndex < 0 || toIndex >= metadata.frame_asset_ids.length) return;

    // Swap in frame_asset_ids
    const tempId = metadata.frame_asset_ids[fromIndex];
    metadata.frame_asset_ids[fromIndex] = metadata.frame_asset_ids[toIndex];
    metadata.frame_asset_ids[toIndex] = tempId;

    // Swap in frame_durations_ms
    const tempDuration = metadata.frame_durations_ms[fromIndex];
    metadata.frame_durations_ms[fromIndex] = metadata.frame_durations_ms[toIndex];
    metadata.frame_durations_ms[toIndex] = tempDuration;

    handleSave();
  }

  async function addFrames(assetIds: string[]) {
    const fps = metadata.default_fps || 12;
    const frameDuration = Math.round(1000 / fps);

    for (const id of assetIds) {
      if (!metadata.frame_asset_ids.includes(id)) {
        metadata.frame_asset_ids.push(id);
        metadata.frame_durations_ms.push(frameDuration);
      }
    }
    metadata.total_duration_ms = metadata.frame_durations_ms.reduce((a, b) => a + b, 0);
    handleSave();
  }

  async function handleExport() {
    try {
      await assetStore.exportAnimation({
        animation_id: animation.id,
        project_id: projectId,
      });
      alert('Export job created! Check the job panel for progress.');
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  async function handleDelete() {
    if (!confirm('Delete this animation? The frame assets will not be deleted.')) return;
    try {
      await assetStore.deleteAnimation(animation.id);
      onclose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }
</script>

<div class="flex flex-col h-full">
  <!-- Header -->
  <div class="flex items-center justify-between px-6 py-4 border-b border-[var(--color-surface)]">
    <div class="flex items-center gap-4">
      {#if editingName}
        <input
          type="text"
          bind:value={nameInput}
          onblur={handleNameSave}
          onkeydown={(e) => e.key === 'Enter' && handleNameSave()}
          class="px-2 py-1 bg-[var(--color-surface)] border border-[var(--color-accent)] rounded text-lg font-bold"
        />
      {:else}
        <h2 class="text-xl font-bold cursor-pointer hover:text-[var(--color-accent)]" onclick={() => { editingName = true; nameInput = metadata.name; }}>
          {metadata.name} (click to edit)
        </h2>
      {/if}
      <span class="text-sm text-[var(--color-text-muted)]">
        {metadata.frame_asset_ids.length} frames | {metadata.total_duration_ms}ms
      </span>
    </div>
    <div class="flex items-center gap-2">
      <button
        onclick={handleExport}
        class="px-3 py-1.5 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg text-sm font-medium"
      >
        Export
      </button>
      <button
        onclick={handleDelete}
        class="px-3 py-1.5 bg-red-500/20 hover:bg-red-500/30 text-red-400 rounded-lg text-sm font-medium"
      >
        Delete
      </button>
      <button
        onclick={onclose}
        class="px-3 py-1.5 bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 rounded-lg text-sm font-medium"
      >
        Close
      </button>
    </div>
  </div>

  {#if error}
    <div class="mx-6 mt-3 p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
      {error}
    </div>
  {/if}

  <!-- Main content -->
  <div class="flex-1 flex overflow-hidden">
    <!-- Timeline strip -->
    <div class="flex-1 flex flex-col overflow-hidden">
      <div class="px-6 py-3 border-b border-[var(--color-surface)]">
        <div class="flex items-center justify-between mb-2">
          <h3 class="font-medium">Timeline</h3>
          <div class="flex items-center gap-2">
            <span class="text-sm text-[var(--color-text-muted)]">Add frames:</span>
            <select
              class="px-2 py-1 bg-[var(--color-surface)] rounded text-sm"
              onchange={(e) => {
                const options = Array.from(e.currentTarget.selectedOptions);
                const ids = options.map(o => o.value);
                addFrames(ids);
                e.currentTarget.value = '';
              }}
            >
              <option value="">Select frames...</option>
              {#each availableFrames as asset}
                {#if !metadata.frame_asset_ids.includes(asset.id)}
                  <option value={asset.id}>{asset.name}</option>
                {/if}
              {/each}
            </select>
          </div>
        </div>
      </div>

      <!-- Frame strip -->
      <div class="flex-1 overflow-x-auto p-4">
        <div class="flex gap-2 min-h-[120px]">
          {#each framesWithUrls as frame, index (frame.id)}
            <div class="flex flex-col items-center shrink-0">
              <div class="relative group">
                <div class="w-20 h-20 rounded-lg overflow-hidden bg-[var(--color-surface)] border-2 border-[var(--color-surface)] hover:border-[var(--color-accent)] transition-colors">
                  {#if frame.url}
                    <img src={frame.url} alt={frame.name} class="w-full h-full object-cover" />
                  {:else}
                    <div class="w-full h-full flex items-center justify-center text-[var(--color-text-muted)] text-xs">
                      {frame.id.slice(0, 8)}
                    </div>
                  {/if}
                </div>
                <!-- Frame controls overlay -->
                <div class="absolute -top-1 -right-1 flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
                  <button
                    onclick={() => moveFrame(index, index - 1)}
                    disabled={index === 0}
                    class="p-1 bg-[var(--color-surface)] rounded text-xs hover:bg-[var(--color-accent)] disabled:opacity-30"
                    title="Move left"
                  >
                    ←
                  </button>
                  <button
                    onclick={() => removeFrame(frame.id)}
                    class="p-1 bg-red-500 rounded text-xs hover:bg-red-400"
                    title="Remove"
                  >
                    ×
                  </button>
                  <button
                    onclick={() => moveFrame(index, index + 1)}
                    disabled={index === framesWithUrls.length - 1}
                    class="p-1 bg-[var(--color-surface)] rounded text-xs hover:bg-[var(--color-accent)] disabled:opacity-30"
                    title="Move right"
                  >
                    →
                  </button>
                </div>
              </div>
              <!-- Duration input -->
              <div class="mt-1 flex items-center gap-1">
                <input
                  type="number"
                  value={frame.duration_ms}
                  onchange={(e) => handleDurationChange(frame.id, e.currentTarget.value)}
                  class="w-16 px-1 py-0.5 text-xs text-center bg-[var(--color-surface)] rounded border border-transparent hover:border-[var(--color-accent)] focus:border-[var(--color-accent)] outline-none"
                  min="1"
                />
                <span class="text-xs text-[var(--color-text-muted)]">ms</span>
              </div>
              <span class="text-xs text-[var(--color-text-muted)] mt-0.5">{index + 1}</span>
            </div>
          {/each}

          {#if framesWithUrls.length === 0}
            <div class="flex items-center justify-center w-full text-[var(--color-text-muted)]">
              No frames yet. Add frames from the dropdown above.
            </div>
          {/if}
        </div>
      </div>
    </div>

    <!-- Canvas player -->
    <div class="w-96 border-l border-[var(--color-surface)] p-4 overflow-y-auto">
      <h3 class="font-medium mb-4">Preview</h3>
      <CanvasPlayer
        animationId={animation.id}
        {metadata}
        frameUrls={new Map(framesWithUrls.map(f => [f.id, f.url]))}
      />
    </div>
  </div>
</div>
