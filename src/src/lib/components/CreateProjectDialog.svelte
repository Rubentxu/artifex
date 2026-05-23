<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { createProject, listProjects } from '$lib/api/projects';
  import type { ProjectResponse } from '$lib/types';
  import { open as openDialog } from '@tauri-apps/plugin-dialog';
  import Dialog from '$lib/components/ui/Dialog.svelte';
  import Button from '$lib/components/ui/Button.svelte';

  interface Props {
    open: boolean;
    onClose: () => void;
    onProjectCreated: (project: ProjectResponse) => void;
  }

  let { open, onClose, onProjectCreated }: Props = $props();

  let name = $state('');
  let path = $state('');
  let error = $state<string | null>(null);
  let loading = $state(false);

  async function handleBrowse() {
    try {
      const selected = await openDialog({
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
      handleClose();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      loading = false;
    }
  }

  function handleClose() {
    name = '';
    path = '';
    error = null;
    loading = false;
    onClose();
  }
</script>

<Dialog
  {open}
  onClose={handleClose}
  title="Create New Project"
  width="md"
  bordered
  {error}
>
  <form onsubmit={(e) => { e.preventDefault(); handleSubmit(); }} class="space-y-4">
    <!-- Name -->
    <div>
      <label for="project-name" class="block text-sm font-medium mb-1">
        Project Name <span class="text-[var(--color-destructive)]">*</span>
      </label>
      <input
        id="project-name"
        type="text"
        bind:value={name}
        placeholder="My Awesome Game"
        class="w-full px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)]"
        maxlength="128"
        required
      />
    </div>

    <!-- Path -->
    <div>
      <label for="project-path" class="block text-sm font-medium mb-1">
        Project Path <span class="text-[var(--color-destructive)]">*</span>
      </label>
      <div class="flex gap-2">
        <input
          id="project-path"
          type="text"
          bind:value={path}
          placeholder="/home/user/projects/mygame"
          class="flex-1 px-3 py-2 rounded-sm bg-[var(--color-canvas)] border border-[var(--color-surface)] focus:border-[var(--color-neon)] focus:shadow-[var(--glow-neon)] focus:outline-none text-[var(--color-text)] placeholder:text-[var(--color-muted)]"
          required
        />
        <button
          type="button"
          onclick={handleBrowse}
          class="px-3 py-2 rounded-sm bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors font-medium"
        >
          Browse
        </button>
      </div>
    </div>
  </form>

  {#snippet footer()}
    <Button variant="ghost" onclick={handleClose} disabled={loading}>
      Cancel
    </Button>
    <Button onclick={handleSubmit} disabled={loading}>
      {loading ? 'Creating...' : 'Create Project'}
    </Button>
  {/snippet}
</Dialog>
