<script lang="ts">
  import { testProviderConnection, setCredential, deleteCredential } from '$lib/api/model-config';

  interface Props {
    providerId: string;
    providerName: string;
    hasCredential?: boolean;
    onSave?: (providerId: string, apiKey: string) => void;
    onDelete?: (providerId: string) => void;
  }

  let { providerId, providerName, hasCredential = false, onSave, onDelete }: Props = $props();

  let apiKey = $state('');
  let showKey = $state(false);
  let testing = $state(false);
  let saving = $state(false);
  let deleting = $state(false);
  let saveError = $state<string | null>(null);
  let deleteError = $state<string | null>(null);

  async function handleTest() {
    testing = true;
    saveError = null;
    try {
      const result = await testProviderConnection(providerId);
      if (result) {
        alert(`Connection to ${providerName} successful!`);
      } else {
        alert(`Connection to ${providerName} failed. Check your API key.`);
      }
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    } finally {
      testing = false;
    }
  }

  async function handleSave() {
    if (!apiKey.trim()) {
      saveError = 'API key cannot be empty';
      return;
    }

    saveError = null;
    saving = true;
    try {
      await setCredential(providerId, apiKey);
      onSave?.(providerId, apiKey);
      apiKey = '';
      alert(`API key for ${providerName} saved successfully!`);
    } catch (e) {
      saveError = e instanceof Error ? e.message : String(e);
    } finally {
      saving = false;
    }
  }

  async function handleDelete() {
    if (!confirm(`Are you sure you want to delete the API key for ${providerName}?`)) {
      return;
    }

    deleteError = null;
    deleting = true;
    try {
      await deleteCredential(providerId);
      onDelete?.(providerId);
      alert(`API key for ${providerName} deleted successfully!`);
    } catch (e) {
      deleteError = e instanceof Error ? e.message : String(e);
    } finally {
      deleting = false;
    }
  }
</script>

<div class="space-y-3 p-4 bg-[var(--color-surface)] rounded-lg">
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-2">
      <span class="font-medium text-[var(--color-text)]">{providerName}</span>
      {#if hasCredential}
        <span class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-green-500/20 text-green-400">
          <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
          </svg>
          Configured
        </span>
      {:else}
        <span class="flex items-center gap-1 text-xs px-2 py-0.5 rounded-full bg-red-500/20 text-red-400">
          <svg class="w-3 h-3" fill="currentColor" viewBox="0 0 20 20">
            <path fill-rule="evenodd" d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z" clip-rule="evenodd"/>
          </svg>
          Not configured
        </span>
      {/if}
    </div>
  </div>

  <div class="relative">
    <input
      type={showKey ? 'text' : 'password'}
      bind:value={apiKey}
      placeholder="Enter API key..."
      class="w-full px-3 py-2 pr-20 bg-[var(--color-panel)] border border-[var(--color-surface)] rounded-lg text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-accent)]"
    />
    <button
      type="button"
      onclick={() => (showKey = !showKey)}
      class="absolute right-2 top-1/2 -translate-y-1/2 p-1.5 text-[var(--color-text-muted)] hover:text-[var(--color-text)]"
      title={showKey ? 'Hide' : 'Show'}
    >
      {#if showKey}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
        </svg>
      {:else}
        <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
        </svg>
      {/if}
    </button>
  </div>

  {#if saveError}
    <p class="text-sm text-red-400">{saveError}</p>
  {/if}

  {#if deleteError}
    <p class="text-sm text-red-400">{deleteError}</p>
  {/if}

  <div class="flex gap-2">
    {#if hasCredential}
      <button
        type="button"
        onclick={handleDelete}
        disabled={deleting}
        class="flex-1 px-3 py-1.5 text-sm bg-red-500/20 hover:bg-red-500/30 text-red-400 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {deleting ? 'Deleting...' : 'Delete'}
      </button>
    {/if}
    <button
      type="button"
      onclick={handleTest}
      disabled={testing || (!hasCredential && !apiKey.trim())}
      class="flex-1 px-3 py-1.5 text-sm bg-[var(--color-panel)] hover:bg-[var(--color-panel)]/80 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {testing ? 'Testing...' : 'Test Connection'}
    </button>
    <button
      type="button"
      onclick={handleSave}
      disabled={saving || !apiKey.trim()}
      class="flex-1 px-3 py-1.5 text-sm bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
    >
      {saving ? 'Saving...' : 'Save'}
    </button>
  </div>
</div>
