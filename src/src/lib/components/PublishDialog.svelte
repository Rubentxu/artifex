<script lang="ts">
  import { assetStore } from '$lib/stores/asset';

  interface Props {
    open: boolean;
    projectId: string;
    projectName: string;
    onclose: () => void;
  }

  let { open, projectId, projectName, onclose }: Props = $props();

  let includeHtmlGallery = $state(true);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let success = $state<{ path: string; size: string; count: number } | null>(null);

  async function handleExport() {
    error = null;
    success = null;
    loading = true;

    try {
      const response = await assetStore.exportProject({
        projectId,
        includeHtmlGallery,
      });

      // Format file size
      let sizeStr: string;
      if (response.fileSizeBytes < 1024) {
        sizeStr = `${response.fileSizeBytes} bytes`;
      } else if (response.fileSizeBytes < 1024 * 1024) {
        sizeStr = `${(response.fileSizeBytes / 1024).toFixed(1)} KB`;
      } else {
        sizeStr = `${(response.fileSizeBytes / (1024 * 1024)).toFixed(1)} MB`;
      }

      success = {
        path: response.outputPath,
        size: sizeStr,
        count: response.assetCount,
      };
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  async function handleOpenItchIo() {
    try {
      await assetStore.openItchIo();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function handleClose() {
    includeHtmlGallery = true;
    loading = false;
    error = null;
    success = null;
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
        <h2 class="text-lg font-semibold">Publish Project</h2>
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
      <div class="p-4 space-y-4">
        {#if error}
          <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
            {error}
          </div>
        {/if}

        {#if success}
          <div class="p-3 rounded-lg bg-green-500/20 border border-green-500/50 text-green-400 text-sm">
            <p class="font-medium mb-1">Export successful!</p>
            <p class="text-sm">Assets: {success.count}</p>
            <p class="text-sm">Size: {success.size}</p>
            <p class="text-sm mt-1 break-all">Saved to: {success.path}</p>
          </div>
        {/if}

        <div class="space-y-4">
          <div class="p-3 rounded-lg bg-[var(--color-surface)] text-sm">
            <p class="text-[var(--color-text-muted)]">Project:</p>
            <p class="font-medium">{projectName}</p>
          </div>

          <!-- Include HTML Gallery checkbox -->
          <label class="flex items-center gap-3 cursor-pointer">
            <input
              type="checkbox"
              bind:checked={includeHtmlGallery}
              class="w-5 h-5 rounded border-[var(--color-surface)] bg-[var(--color-canvas)] text-[var(--color-accent)] focus:ring-[var(--color-accent)] focus:ring-offset-0"
            />
            <div>
              <p class="font-medium">Include HTML Gallery</p>
              <p class="text-sm text-[var(--color-text-muted)]">Generate an index.html file to preview all assets</p>
            </div>
          </label>

          <!-- Export Contents Info -->
          <div class="text-sm text-[var(--color-text-muted)]">
            <p>The export package will include:</p>
            <ul class="list-disc list-inside mt-2 space-y-1">
              <li>All project assets organized by type</li>
              <li>manifest.json with asset metadata</li>
              {#if includeHtmlGallery}
                <li>index.html gallery (self-contained, no external dependencies)</li>
              {/if}
            </ul>
          </div>
        </div>

        <!-- Actions -->
        <div class="flex justify-between gap-2 pt-2">
          <button
            onclick={handleOpenItchIo}
            class="flex items-center gap-2 px-4 py-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors font-medium"
          >
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
            </svg>
            Open itch.io
          </button>
          <div class="flex gap-2">
            <button
              type="button"
              onclick={handleClose}
              class="px-4 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors"
              disabled={loading}
            >
              {success ? 'Close' : 'Cancel'}
            </button>
            {#if !success}
              <button
                type="button"
                onclick={handleExport}
                class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50"
                disabled={loading}
              >
                {loading ? 'Exporting...' : 'Export ZIP'}
              </button>
            {/if}
          </div>
        </div>
      </div>
    </div>
  </div>
{/if}
