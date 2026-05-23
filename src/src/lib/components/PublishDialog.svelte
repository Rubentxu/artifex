<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Publish Project"
  width="lg"
  bordered
  {error}
>
  <div class="space-y-4">
    {#if success}
      <div class="p-3 rounded-sm bg-[var(--color-neon)]/20 border border-[var(--color-neon)]/50 text-[var(--color-neon)] text-sm">
        <p class="font-medium mb-1">Export successful!</p>
        <p class="text-sm">Assets: {success.count}</p>
        <p class="text-sm">Size: {success.size}</p>
        <p class="text-sm mt-1 break-all">Saved to: {success.path}</p>
      </div>
    {/if}

    <div class="p-3 rounded-sm bg-[var(--color-surface)] text-sm">
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

  {#snippet footer()}
    <Button onclick={handleOpenItchIo}>
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" />
      </svg>
      Open itch.io
    </Button>
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      {success ? 'Close' : 'Cancel'}
    </Button>
    {#if !success}
      <Button onclick={handleExport} disabled={loading}>
        {loading ? 'Exporting...' : 'Export ZIP'}
      </Button>
    {/if}
  {/snippet}
</Dialog>
