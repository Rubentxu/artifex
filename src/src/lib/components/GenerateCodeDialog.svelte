<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateCodeRequest, CodeEngine, CodeTemplate, CodeFileOutput } from '$lib/types/asset';
  import * as assetApi from '$lib/api/assets';
  import CodePreviewPanel from './CodePreviewPanel.svelte';

  interface Props {
    open: boolean;
    projectId: string;
    onclose: () => void;
  }

  let { open, projectId, onclose }: Props = $props();

  let engine = $state<CodeEngine>('godot');
  let selectedTemplateId = $state<string>('');
  let prompt = $state('');
  let temperature = $state(0.25);
  let maxTokens = $state(4096);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let templates = $state<CodeTemplate[]>([]);
  let loadingTemplates = $state(false);

  // State for showing preview after generation
  let showPreview = $state(false);
  let generatedFiles = $state<CodeFileOutput[]>([]);
  let unlistenJobCompleted: (() => void) | null = null;

  // Load templates when engine changes
  $effect(() => {
    if (open && engine) {
      loadTemplates(engine);
    }
  });

  // Set up job-completed listener when dialog opens
  $effect(() => {
    if (open) {
      setupJobListener();
    } else {
      cleanupJobListener();
    }
  });

  onDestroy(() => {
    cleanupJobListener();
  });

  function cleanupJobListener() {
    if (unlistenJobCompleted) {
      unlistenJobCompleted();
      unlistenJobCompleted = null;
    }
  }

  async function setupJobListener() {
    cleanupJobListener();
    try {
      const { listen } = await import('@tauri-apps/api/event');
      unlistenJobCompleted = await listen<{ job_id: string; asset_ids: string[] }>('job-completed', (event) => {
        // When any code generation job completes, refresh assets
        // The asset list will be refreshed by the assets page listener
        console.log('Code generation job completed:', event.payload);
      });
    } catch (e) {
      console.warn('Failed to listen for job-completed event:', e);
    }
  }

  async function loadTemplates(engineType: CodeEngine) {
    loadingTemplates = true;
    try {
      templates = await assetApi.listCodeTemplates(engineType);
      // Reset template selection if current one isn't available for new engine
      if (selectedTemplateId && !templates.find(t => t.id === selectedTemplateId)) {
        selectedTemplateId = '';
      }
    } catch (e) {
      console.error('Failed to load templates:', e);
    } finally {
      loadingTemplates = false;
    }
  }

  async function handleGenerate() {
    error = null;

    if (!prompt.trim()) {
      error = 'Prompt is required';
      return;
    }

    loading = true;
    showPreview = false;
    generatedFiles = [];
    try {
      const request: GenerateCodeRequest = {
        projectId: projectId,
        engine,
        prompt: prompt.trim(),
        templateId: selectedTemplateId || undefined,
        temperature,
        maxTokens: maxTokens,
      };
      const jobId = await assetStore.generateCode(request);
      console.log('Code generation job started:', jobId);

      // Refresh assets to show the new code asset
      await assetStore.loadAssets(projectId);

      // Show preview state (user can also see it in the asset list)
      showPreview = true;
      // Note: We don't close the dialog - user can see the preview state
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    prompt = '';
    selectedTemplateId = '';
    temperature = 0.25;
    maxTokens = 4096;
    error = null;
    showPreview = false;
    generatedFiles = [];
    onclose();
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      handleClose();
    }
  }

  // Filter templates by current engine (templates are already filtered by engine from API)
  let filteredTemplates = $derived(templates);
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
      class="bg-[var(--color-panel)] rounded-xl shadow-2xl w-full max-w-4xl mx-4 max-h-[90vh] flex flex-col border border-[var(--color-surface)]"
      onclick={(e) => e.stopPropagation()}
    >
      <!-- Header -->
      <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)]">
        <h2 class="text-lg font-semibold">{showPreview ? 'Code Generated' : 'Generate Code'}</h2>
        <button
          onclick={handleClose}
          class="p-1 rounded hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text-muted)]"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
          </svg>
        </button>
      </div>

      {#if showPreview}
        <!-- Preview Mode: Show generated code assets -->
        <div class="flex-1 overflow-hidden p-4">
          <div class="mb-4 p-3 rounded-lg bg-green-500/20 border border-green-500/50 text-green-400 text-sm">
            Code generation completed! The generated files are now available in your assets. Select them to preview.
          </div>
          <p class="text-sm text-[var(--color-text-muted)] mb-4">
            Generated code assets have been added to your project. Use the asset panel to view and manage them.
          </p>
          <!-- CodePreviewPanel is available for integration when viewing specific assets -->
          <div class="flex justify-end">
            <button
              onclick={handleClose}
              class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium"
            >
              Done
            </button>
          </div>
        </div>
      {:else}
        <!-- Form Mode -->
        <form onsubmit={(e) => { e.preventDefault(); handleGenerate(); }} class="p-4 space-y-4">
          {#if error}
            <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
              {error}
            </div>
          {/if}

          <!-- Engine Selection -->
          <div>
            <label class="block text-sm font-medium mb-2">
              Engine <span class="text-red-400">*</span>
            </label>
            <div class="flex gap-4">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="engine"
                  value="godot"
                  bind:group={engine}
                  class="w-4 h-4 text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                />
                <span class="text-sm">Godot (GDScript)</span>
              </label>
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="radio"
                  name="engine"
                  value="unity"
                  bind:group={engine}
                  class="w-4 h-4 text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                />
                <span class="text-sm">Unity (C#)</span>
              </label>
            </div>
          </div>

          <!-- Template Selection -->
          <div>
            <label for="code-template" class="block text-sm font-medium mb-1">
              Template (optional)
            </label>
            <select
              id="code-template"
              bind:value={selectedTemplateId}
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
              disabled={loadingTemplates}
            >
              <option value="">No template (free-form prompt)</option>
              {#each filteredTemplates as tmpl}
                <option value={tmpl.id}>{tmpl.name}</option>
              {/each}
            </select>
            {#if templates.find(t => t.id === selectedTemplateId)}
              {@const selectedTemplate = templates.find(t => t.id === selectedTemplateId)}
              {#if selectedTemplate}
                <p class="text-xs text-[var(--color-text-muted)] mt-1">
                  {selectedTemplate.description}
                </p>
              {/if}
            {/if}
          </div>

          <!-- Prompt -->
          <div>
            <label for="code-prompt" class="block text-sm font-medium mb-1">
              Prompt <span class="text-red-400">*</span>
            </label>
            <textarea
              id="code-prompt"
              bind:value={prompt}
              placeholder={selectedTemplateId ? "Describe what you want to generate based on the template..." : "e.g., A player controller with dash ability for a 2D platformer..."}
              rows="4"
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] resize-none"
              required
            ></textarea>
          </div>

          <!-- Temperature -->
          <div>
            <label for="code-temperature" class="block text-sm font-medium mb-1">
              Temperature: {temperature}
            </label>
            <input
              id="code-temperature"
              type="range"
              bind:value={temperature}
              min="0"
              max="1"
              step="0.05"
              class="w-full h-2 rounded-lg appearance-none cursor-pointer bg-[var(--color-surface)] accent-[var(--color-accent)]"
            />
            <div class="flex justify-between text-xs text-[var(--color-text-muted)] mt-1">
              <span>0 (Focused)</span>
              <span>1 (Creative)</span>
            </div>
          </div>

          <!-- Max Tokens -->
          <div>
            <label for="code-max-tokens" class="block text-sm font-medium mb-1">
              Max Tokens
            </label>
            <input
              id="code-max-tokens"
              type="number"
              bind:value={maxTokens}
              min="256"
              max="8192"
              step="256"
              class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)]"
            />
          </div>

          <!-- Actions -->
          <div class="flex justify-end gap-2 pt-2">
            <button
              type="button"
              onclick={handleClose}
              class="px-4 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors"
              disabled={loading}
            >
              Cancel
            </button>
            <button
              type="submit"
              class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50"
              disabled={loading || !prompt.trim()}
            >
              {loading ? 'Generating...' : 'Generate'}
            </button>
          </div>
        </form>
      {/if}
    </div>
  </div>
{/if}
