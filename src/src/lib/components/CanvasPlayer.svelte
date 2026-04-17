<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import type { AnimationMetadata } from '$lib/types/asset';

  interface FrameData {
    id: string;
    url: string;
    duration_ms: number;
  }

  interface Props {
    animationId: string;
    metadata: AnimationMetadata;
    frameUrls: Map<string, string>; // asset_id -> file URL
  }

  let { animationId, metadata, frameUrls }: Props = $props();

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let currentFrameIndex = $state(0);
  let isPlaying = $state(false);
  let loop = $state(true);
  let speed = $state(1.0);
  let lastFrameTime = 0;
  let animationFrameId: number | null = null;

  const speedOptions = [0.25, 0.5, 1.0, 2.0, 4.0];

  // Build frame data from metadata
  let frames: FrameData[] = $derived(
    metadata.frame_asset_ids.map((id, index) => ({
      id,
      url: frameUrls.get(id) || '',
      duration_ms: metadata.frame_durations_ms[index] || 100,
    }))
  );

  let currentFrame = $derived(frames[currentFrameIndex]);

  onMount(() => {
    if (canvas) {
      ctx = canvas.getContext('2d');
    }
  });

  onDestroy(() => {
    stopPlayback();
  });

  function startPlayback() {
    if (frames.length === 0) return;
    isPlaying = true;
    lastFrameTime = performance.now();
    playLoop();
  }

  function stopPlayback() {
    isPlaying = false;
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }
    currentFrameIndex = 0;
  }

  function pausePlayback() {
    isPlaying = false;
    if (animationFrameId !== null) {
      cancelAnimationFrame(animationFrameId);
      animationFrameId = null;
    }
  }

  function toggleLoop() {
    loop = !loop;
  }

  function setSpeed(newSpeed: number) {
    speed = newSpeed;
  }

  function playLoop() {
    if (!isPlaying) return;

    const now = performance.now();
    const elapsed = now - lastFrameTime;
    const frameDuration = currentFrame.duration_ms / speed;

    if (elapsed >= frameDuration) {
      lastFrameTime = now;
      advanceFrame();
    }

    animationFrameId = requestAnimationFrame(playLoop);
  }

  function advanceFrame() {
    if (currentFrameIndex < frames.length - 1) {
      currentFrameIndex++;
    } else if (loop) {
      currentFrameIndex = 0;
    } else {
      isPlaying = false;
    }
  }

  // Draw current frame when it changes
  $effect(() => {
    if (ctx && currentFrame) {
      drawFrame(currentFrame.url);
    }
  });

  async function drawFrame(url: string) {
    if (!ctx || !url) return;

    try {
      const img = new Image();
      img.src = url;
      await new Promise((resolve, reject) => {
        img.onload = resolve;
        img.onerror = reject;
      });

      // Clear canvas
      ctx.clearRect(0, 0, canvas.width, canvas.height);

      // Draw image centered and scaled to fit
      const scale = Math.min(canvas.width / img.width, canvas.height / img.height);
      const x = (canvas.width - img.width * scale) / 2;
      const y = (canvas.height - img.height * scale) / 2;

      ctx.drawImage(img, x, y, img.width * scale, img.height * scale);
    } catch (e) {
      console.error('Failed to draw frame:', e);
    }
  }
</script>

<div class="flex flex-col gap-4">
  <!-- Canvas -->
  <div class="relative bg-black rounded-lg overflow-hidden" style="aspect-ratio: 16/9;">
    <canvas
      bind:this={canvas}
      width={512}
      height={288}
      class="w-full h-full"
    ></canvas>

    <!-- Frame indicator -->
    <div class="absolute bottom-2 right-2 px-2 py-1 bg-black/70 rounded text-xs text-white font-mono">
      {currentFrameIndex + 1} / {frames.length}
    </div>
  </div>

  <!-- Controls -->
  <div class="flex items-center justify-center gap-4">
    <!-- Play/Pause -->
    {#if isPlaying}
      <button
        onclick={pausePlayback}
        class="p-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors"
        title="Pause"
      >
        <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
          <path d="M6 4h4v16H6V4zm8 0h4v16h-4V4z"/>
        </svg>
      </button>
    {:else}
      <button
        onclick={startPlayback}
        class="p-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors"
        title="Play"
      >
        <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
          <path d="M8 5v14l11-7z"/>
        </svg>
      </button>
    {/if}

    <!-- Stop -->
    <button
      onclick={stopPlayback}
      class="p-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors"
      title="Stop"
    >
      <svg class="w-6 h-6" fill="currentColor" viewBox="0 0 24 24">
        <path d="M6 6h12v12H6z"/>
      </svg>
    </button>

    <!-- Loop toggle -->
    <button
      onclick={toggleLoop}
      class="p-2 rounded-lg transition-colors {loop ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-surface)]'} hover:opacity-80"
      title="Loop"
    >
      <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
      </svg>
    </button>

    <!-- Speed control -->
    <div class="flex items-center gap-1">
      {#each speedOptions as s}
        <button
          onclick={() => setSpeed(s)}
          class="px-2 py-1 rounded text-xs font-mono transition-colors {speed === s ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-surface)]'} hover:opacity-80"
        >
          {s}×
        </button>
      {/each}
    </div>
  </div>

  <!-- Current frame info -->
  <div class="text-sm text-center text-[var(--color-text-muted)]">
    Frame {currentFrameIndex + 1}: {currentFrame?.duration_ms}ms | Total: {metadata.total_duration_ms}ms
  </div>
</div>
