<script lang="ts">
  import type { PromptTemplateDto } from '$lib/api/model-config';
  import { createPromptTemplate, deletePromptTemplate } from '$lib/api/model-config';

  interface Props {
    templates: PromptTemplateDto[];
    onUpdate?: () => void;
  }

  let { templates, onUpdate }: Props = $props();

  let showCreateForm = $state(false);
  let newName = $state('');
  let newTemplate = $state('');
  let creating = $state(false);
  let error = $state<string | null>(null);

  async function handleCreate() {
    if (!newName.trim() || !newTemplate.trim()) {
      error = 'Name and template are required';
      return;
    }

    creating = true;
    error = null;

    try {
      // Extract variables from template (e.g., {{name}} -> name)
      const variableMatches = newTemplate.matchAll(/\{\{(\w+)\}\}/g);
      const variables = [...new Set([...variableMatches].map(m => m[1]))];

      await createPromptTemplate(newName.trim(), newTemplate.trim());

      newName = '';
      newTemplate = '';
      showCreateForm = false;
      onUpdate?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      creating = false;
    }
  }

  async function handleDelete(id: string) {
    if (!confirm('Are you sure you want to delete this template?')) return;

    try {
      await deletePromptTemplate(id);
      onUpdate?.();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  function renderTemplate(template: string, variables: string[]): string {
    let result = template;
    for (const v of variables) {
      result = result.replace(new RegExp(`\\{\\{${v}\\}\\}`, 'g'), `[${v}]`);
    }
    return result;
  }
</script>

<div class="space-y-4">
  <div class="flex items-center justify-between">
    <h3 class="text-lg font-semibold text-[var(--color-text)]">Prompt Templates</h3>
    <button
      type="button"
      onclick={() => (showCreateForm = !showCreateForm)}
      class="px-3 py-1.5 text-sm bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 rounded-lg transition-colors"
    >
      {showCreateForm ? 'Cancel' : '+ New Template'}
    </button>
  </div>

  {#if error}
    <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
      {error}
    </div>
  {/if}

  {#if showCreateForm}
    <div class="p-4 bg-[var(--color-surface)] rounded-lg space-y-3">
      <div>
        <label for="template-name" class="block text-sm font-medium text-[var(--color-text)] mb-1">Name</label>
        <input
          id="template-name"
          type="text"
          bind:value={newName}
          placeholder="e.g., NPC Dialogue"
          class="w-full px-3 py-2 bg-[var(--color-panel)] border border-[var(--color-surface)] rounded-lg text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-accent)]"
        />
      </div>

      <div>
        <label for="template-text" class="block text-sm font-medium text-[var(--color-text)] mb-1">
          Template (use {"{{variable}}"} for variables)
        </label>
        <textarea
          id="template-text"
          bind:value={newTemplate}
          placeholder={"e.g., You are {{character}}, in a {{setting}}. Say: {{dialogue}}"}
          rows="4"
          class="w-full px-3 py-2 bg-[var(--color-panel)] border border-[var(--color-surface)] rounded-lg text-[var(--color-text)] placeholder:text-[var(--color-text-muted)] focus:outline-none focus:border-[var(--color-accent)] resize-none"
        ></textarea>
      </div>

      <button
        type="button"
        onclick={handleCreate}
        disabled={creating}
        class="w-full px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 rounded-lg transition-colors disabled:opacity-50"
      >
        {creating ? 'Creating...' : 'Create Template'}
      </button>
    </div>
  {/if}

  {#if templates.length === 0}
    <p class="text-sm text-[var(--color-text-muted)]">No prompt templates yet.</p>
  {:else}
    <div class="space-y-2">
      {#each templates as template (template.id)}
        <div class="p-4 bg-[var(--color-surface)] rounded-lg">
          <div class="flex items-start justify-between gap-4">
            <div class="flex-1 min-w-0">
              <div class="font-medium text-[var(--color-text)]">{template.name}</div>
              <div class="mt-1 text-sm text-[var(--color-text-muted)] line-clamp-2">
                {renderTemplate(template.template_text, template.variables)}
              </div>
              {#if template.variables.length > 0}
                <div class="mt-2 flex flex-wrap gap-1">
                  {#each template.variables as variable}
                    <span class="text-xs px-2 py-0.5 rounded-full bg-[var(--color-panel)] text-[var(--color-text-muted)]">
                      {variable}
                    </span>
                  {/each}
                </div>
              {/if}
            </div>

            <button
              type="button"
              onclick={() => handleDelete(template.id)}
              class="shrink-0 p-1.5 text-red-400 hover:text-red-300 transition-colors"
              title="Delete template"
            >
              <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
              </svg>
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>
