<script lang="ts">
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateMaterialRequest } from '$lib/types/asset';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  interface Props {
    open: boolean;
    projectId: string;
    assetId: string;
    onclose: () => void;
  }

  let { open, projectId, assetId, onclose }: Props = $props();

  let loading = $state(false);
  let error = $state<string | null>(null);

  async function handleGenerateMaterial() {
    error = null;
    loading = true;
    try {
      const request: GenerateMaterialRequest = {
        project_id: projectId,
        asset_id: assetId,
      };
      await assetStore.generateMaterial(request);
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
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Generate PBR Material"
  {error}
  width="md"
  bordered
>
  <p class="text-sm text-[var(--color-text-muted)]">
    Generate PBR material maps (basecolor, normal, roughness, metalness, height) from the selected image using AI. The generated material will be saved as a new asset.
  </p>

  {#if loading}
    <div class="flex items-center justify-center py-4">
      <div class="animate-spin w-6 h-6 border-2 border-[var(--color-neon)] border-t-transparent rounded-full"></div>
      <span class="ml-2 text-sm">Generating material...</span>
    </div>
  {/if}

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleGenerateMaterial} disabled={loading}>
      {loading ? 'Generating...' : 'Generate Material'}
    </Button>
  {/snippet}
</Dialog>
