<script lang="ts">
  import { onMount } from 'svelte';
  import { projectStore, selectedProject } from '$lib/stores/project';
  import ProjectCard from '$lib/components/ProjectCard.svelte';
  import CreateProjectDialog from '$lib/components/CreateProjectDialog.svelte';
  import type { ProjectResponse } from '$lib/types';

  let showCreateDialog = $state(false);
  let contextMenuProject: ProjectResponse | null = $state(null);
  let contextMenuPosition = $state({ x: 0, y: 0 });
  let showContextMenu = $state(false);

  onMount(() => {
    projectStore.loadProjects();

    // Listen for create project event from sidebar
    const handleOpenCreateProject = () => {
      showCreateDialog = true;
    };
    window.addEventListener('open-create-project', handleOpenCreateProject);

    // Close context menu on click outside
    const handleClick = () => {
      showContextMenu = false;
    };
    window.addEventListener('click', handleClick);

    return () => {
      window.removeEventListener('open-create-project', handleOpenCreateProject);
      window.removeEventListener('click', handleClick);
    };
  });

  function handleProjectCreated(project: ProjectResponse) {
    projectStore.selectProject(project.id);
    projectStore.loadProjects();
  }

  function handleContextMenu(event: MouseEvent, project: ProjectResponse) {
    contextMenuProject = project;
    contextMenuPosition = { x: event.clientX, y: event.clientY };
    showContextMenu = true;
  }

  async function handleRename() {
    if (!contextMenuProject) return;
    const newName = prompt('Enter new name:', contextMenuProject.name);
    if (newName && newName !== contextMenuProject.name) {
      try {
        await projectStore.renameProject(contextMenuProject.id, newName);
        if ($selectedProject?.id === contextMenuProject.id) {
          projectStore.selectProject(contextMenuProject.id);
        }
      } catch (e) {
        console.error('Failed to rename project:', e);
        alert('Failed to rename project: ' + e);
      }
    }
    showContextMenu = false;
  }

  async function handleArchive() {
    if (!contextMenuProject) return;
    if (!confirm(`Archive project "${contextMenuProject.name}"?`)) return;
    try {
      await projectStore.archiveProject(contextMenuProject.id);
    } catch (e) {
      console.error('Failed to archive project:', e);
      alert('Failed to archive project: ' + e);
    }
    showContextMenu = false;
  }

  async function handleDelete() {
    if (!contextMenuProject) return;
    alert('Hard delete is not implemented. Use archive instead.');
    showContextMenu = false;
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <header class="flex items-center justify-between px-6 py-4 border-b border-[var(--color-surface)]">
    <h1 class="text-2xl font-bold">Projects</h1>
    <button
      onclick={() => (showCreateDialog = true)}
      class="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 rounded-lg transition-colors font-medium"
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
      </svg>
      New Project
    </button>
  </header>

  <!-- Project Grid -->
  <main class="flex-1 overflow-y-auto p-6">
    {#if $projectStore.projects.length > 0}
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        {#each $projectStore.projects as project (project.id)}
          <ProjectCard {project} onContextMenu={handleContextMenu} />
        {/each}
      </div>
    {:else}
      <div class="flex flex-col items-center justify-center h-full text-center">
        <div class="w-24 h-24 rounded-full bg-[var(--color-panel)] flex items-center justify-center mb-4">
          <svg class="w-12 h-12 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
        </div>
        <h2 class="text-xl font-semibold mb-2">No projects yet</h2>
        <p class="text-[var(--color-text-muted)] mb-4">Create your first project to get started</p>
        <button
          onclick={() => (showCreateDialog = true)}
          class="flex items-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 rounded-lg transition-colors font-medium"
        >
          <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
          </svg>
          New Project
        </button>
      </div>
    {/if}
  </main>
</div>

<!-- Context Menu -->
{#if showContextMenu && contextMenuProject}
  <div
    class="fixed z-50 bg-[var(--color-panel)] border border-[var(--color-surface)] rounded-lg shadow-xl py-1 min-w-[160px]"
    style="left: {contextMenuPosition.x}px; top: {contextMenuPosition.y}px"
  >
    <button
      onclick={handleRename}
      class="w-full px-4 py-2 text-left hover:bg-[var(--color-surface)] transition-colors flex items-center gap-2"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
      </svg>
      Rename
    </button>
    <button
      onclick={handleArchive}
      class="w-full px-4 py-2 text-left hover:bg-[var(--color-surface)] transition-colors flex items-center gap-2"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4" />
      </svg>
      Archive
    </button>
    <button
      onclick={handleDelete}
      class="w-full px-4 py-2 text-left hover:bg-[var(--color-surface)] transition-colors flex items-center gap-2 text-red-400"
    >
      <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
      </svg>
      Delete
    </button>
  </div>
{/if}

<!-- Create Project Dialog -->
{#if showCreateDialog}
  <CreateProjectDialog
    onClose={() => (showCreateDialog = false)}
    onProjectCreated={handleProjectCreated}
  />
{/if}
