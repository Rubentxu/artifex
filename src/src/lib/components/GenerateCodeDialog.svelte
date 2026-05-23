<script lang="ts">
  import { onDestroy } from 'svelte';
  import { assetStore } from '$lib/stores/asset';
  import type { GenerateCodeRequest, CodeEngine, CodeTemplate, CodeFileOutput } from '$lib/types/asset';
  import * as assetApi from '$lib/api/assets';
  import CodePreviewPanel from './CodePreviewPanel.svelte';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

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

  let showPreview = $state(false);
  let generatedFiles = $state<CodeFileOutput[]>([]);
  let unlistenJobCompleted: (() => void) | null = null;

  $effect(() => {
    if (open && engine) {
      loadTemplates(engine);
    }
  });

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
      await assetStore.loadAssets(projectId);
      showPreview = true;
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

  let filteredTemplates = $derived(templates);
</script>

<Dialog
  {open}
  onClose={handleClose}
  title={showPreview ? 'Code Generated' : 'Generate Code'}
  width="4xl"
  bordered
  scrollBody
  {error}
>
  {#if showPreview}
    <!-- Preview Mode -->
    <div class="mb-4 p-3 rounded-sm bg-[var(--color-neon)]/20 border border-[var(--color-neon)]/50 text-[var(--color-neon)] text-sm">
      Code generation completed! The generated files are now available in your assets. Select them to preview.
    </div>
    <p class="text-sm text-[var(--color-text-muted)] mb-4">
      Generated code assets have been added to your project. Use the asset panel to view and manage them.
    </p>
  {:else}
    <!-- Form Mode -->
    <form onsubmit={(e) => { e.preventDefault(); handleGenerate(); }} class="space-y-4">
      <!-- Engine Selection -->
      <div>
        <label class="block text-sm font-medium mb-2">
          Engine <span class="text-[var(--color-destructive)]">*</span>
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
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
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
          Prompt <span class="text-[var(--color-destructive)]">*</span>
        </label>
        <textarea
          id="code-prompt"
          bind:value={prompt}
          placeholder={selectedTemplateId ? "Describe what you want to generate based on the template..." : "e.g., A player controller with dash ability for a 2D platformer..."}
          rows="4"
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)] resize-none"
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
          class="w-full h-2 rounded-sm appearance-none cursor-pointer bg-[var(--color-surface)] accent-[var(--color-accent)]"
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
          class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)]"
        />
      </div>
    </form>
  {/if}

  {#snippet footer()}
    {#if showPreview}
      <Button onclick={handleClose}>
        Done
      </Button>
    {:else}
      <Button variant="ghost" onclick={handleClose} disabled={loading}>
        Cancel
      </Button>
      <Button onclick={handleGenerate} disabled={loading || !prompt.trim()}>
        {loading ? 'Generating...' : 'Generate'}
      </Button>
    {/if}
  {/snippet}
</Dialog>
