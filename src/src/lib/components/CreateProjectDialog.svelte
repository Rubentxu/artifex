<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { createProject, listProjects } from '$lib/api/projects';
  import type { ProjectResponse } from '$lib/types';
  import { open } from '@tauri-apps/plugin-dialog';

  interface Props {
    onClose: () => void;
    onProjectCreated: (project: ProjectResponse) => void;
  }

  let { onClose, onProjectCreated }: Props = $props();

  let name = $state('');
  let path = $state('');
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function handleBrowse() {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: 'Select project directory',
      });
      if (selected) {
        path = selected as string;
      }
    } catch (e) {
      console.error('Failed to open dialog:', e);
    }
  }

  async function handleSubmit() {
    error = null;

    // Validate
    if (!name.trim()) {
      error = 'Project name is required';
      return;
    }
    if (!path.trim()) {
      error = 'Project path is required';
      return;
    }

    loading = true;
    try {
      const project = await createProject(name.trim(), path.trim());
      onProjectCreated(project);
      onClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleKeydown(event: KeyboardEvent) {
    if (event.key === 'Escape') {
      onClose();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<!-- Backdrop -->
<div
  class="fixed inset-0 bg-black/50 flex items-center justify-center z-50"
  onclick={onClose}
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
      <h2 class="text-lg font-semibold">Create New Project</h2>
      <button
        onclick={onClose}
        class="p-1 rounded hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text-muted)]"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>

    <!-- Form -->
    <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="p-4 space-y-4">
      {#if error}
        <div class="p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
          {error}
        </div>
      {/if}

      <!-- Name -->
      <div>
        <label for="project-name" class="block text-sm font-medium mb-1">
          Project Name <span class="text-red-400">*</span>
        </label>
        <input
          id="project-name"
          type="text"
          bind:value={name}
          placeholder="My Awesome Game"
          class="w-full px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)]"
          maxlength="128"
          required
        />
      </div>

      <!-- Path -->
      <div>
        <label for="project-path" class="block text-sm font-medium mb-1">
          Project Path <span class="text-red-400">*</span>
        </label>
        <div class="flex gap-2">
          <input
            id="project-path"
            type="text"
            bind:value={path}
            placeholder="/home/user/projects/mygame"
            class="flex-1 px-3 py-2 rounded-lg bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-accent)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-text-muted)]"
            required
          />
          <button
            type="button"
            onclick={handleBrowse}
            class="px-3 py-2 rounded-lg bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors font-medium"
          >
            Browse
          </button>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex justify-end gap-2 pt-2">
        <button
          type="button"
          onclick={onClose}
          class="px-4 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors"
          disabled={loading}
        >
          Cancel
        </button>
        <button
          type="submit"
          class="px-4 py-2 rounded-lg bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 transition-colors font-medium disabled:opacity-50"
          disabled={loading}
        >
          {loading ? 'Creating...' : 'Create Project'}
        </button>
      </div>
    </form>
  </div>
</div>
