<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { RemoveBackgroundRequest } from '$lib/types/asset';

  interface Props {
    open: boolean;
    projectId: string;
    assetId: string;
    onclose: () => void;
  }

  let { open, projectId, assetId, onclose }: Props = $props();

  let loading = $state(false);
  let error = $state<string | null>(null);

  async function handleRemoveBackground() {
    error = null;
    loading = true;
    try {
      const request: RemoveBackgroundRequest = {
        project_id: projectId,
        asset_id: assetId,
      };
      await assetStore.removeBackground(request);
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
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-md mx-4 border border-[var(--color-surface)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)]">
        <h2 class="text-lg font-semibold">Remove Background</h2>
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

        <p class="text-sm text-[var(--color-text-muted)]">
          This will remove the background from the selected image using AI. The processed image will be saved as a new asset.
        </p>

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
          onclick={handleRemoveBackground}
          class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50"
          disabled={loading}
        >
          {loading ? 'Processing...' : 'Remove Background'}
        </button>
      </div>
    </div>
  </div>
{/if}