<script lang="ts">
  import { page } from '$app/state';
  import { sidebarCollapsed, toggleSidebar } from '$lib/stores/ui';
  import { projectStore, selectedProject } from '$lib/stores/project';
  import { identityStore } from '$lib/stores/identity';
  import type { ProjectResponse } from '$lib/types';
  import { onMount } from 'svelte';

  function handleProjectClick(project: ProjectResponse) {
    projectStore.selectProject(project.id);
  }

  function handleNewProject() {
    // Dispatch custom event for parent to handle
    window.dispatchEvent(new CustomEvent('open-create-project'));
  }

  // Load identity on mount
  onMount(() => {
    identityStore.loadIdentity();
  });
</script>

<aside
  class="h-full bg-[var(--color-panel)] border-r border-[var(--color-surface)] flex flex-col transition-all duration-200"
  class:w-64={!$sidebarCollapsed}
  class:w-16={$sidebarCollapsed}
>
  <!-- Header with logo and collapse toggle -->
  <div class="flex items-center justify-between p-4 border-b border-[var(--color-surface)]">
    {#if !$sidebarCollapsed}
      <div class="flex items-center gap-2">
        <div class="w-8 h-8 bg-[var(--color-accent)] rounded-lg flex items-center justify-center font-bold text-white">
          A
        </div>
        <span class="font-semibold text-lg">Artifex</span>
        {#if $identityStore.tier === 'pro'}
          <span class="px-2 py-0.5 text-xs font-bold uppercase bg-purple-500/20 text-purple-400 rounded">
            PRO
          </span>
        {/if}
      </div>
    {:else}
      <div class="w-8 h-8 bg-[var(--color-accent)] rounded-lg flex items-center justify-center font-bold text-white mx-auto">
        A
      </div>
    {/if}
    <button
      onclick={toggleSidebar}
      class="p-1.5 rounded hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text-muted)]"
      title={$sidebarCollapsed ? 'Expand sidebar' : 'Collapse sidebar'}
    >
      <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        {#if $sidebarCollapsed}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 5l7 7-7 7M5 5l7 7-7 7" />
        {:else}
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
        {/if}
      </svg>
    </button>
  </div>

  {#if !$sidebarCollapsed}
    <!-- Navigation -->
    <nav class="p-4 space-y-1">
      <div class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider mb-2">
        Navigation
      </div>
      <a href="/" class="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-[var(--color-text)]" class:bg-[var(--color-surface)]={page.url.pathname === '/'} class:hover:bg-[var(--color-surface)]={true}>
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
        </svg>
        Dashboard
      </a>
      <a href="/" class="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-[var(--color-text)]" class:bg-[var(--color-surface)]={page.url.pathname === '/'} class:hover:bg-[var(--color-surface)]={true}>
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
        </svg>
        Projects
      </a>
      <a href="/assets" class="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-[var(--color-text)]" class:bg-[var(--color-surface)]={page.url.pathname.startsWith('/assets')} class:hover:bg-[var(--color-surface)]={true}>
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        Assets
      </a>
      <a href="/settings" class="flex items-center gap-3 px-3 py-2 rounded-lg transition-colors text-[var(--color-text)]" class:bg-[var(--color-surface)]={page.url.pathname.startsWith('/settings')} class:hover:bg-[var(--color-surface)]={true}>
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
        Settings
      </a>
    </nav>

    <!-- Recent Projects -->
    <div class="flex-1 overflow-y-auto p-4">
      <div class="text-xs font-semibold text-[var(--color-text-muted)] uppercase tracking-wider mb-2">
        Recent Projects
      </div>
      <div class="space-y-1">
        {#each $projectStore.projects as project (project.id)}
          <button
            onclick={() => handleProjectClick(project)}
            class="w-full text-left px-3 py-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors text-sm"
            class:bg-[var(--color-surface)]={$selectedProject?.id === project.id}
          >
            <div class="truncate font-medium">{project.name}</div>
            <div class="truncate text-xs text-[var(--color-text-muted)]">{project.path}</div>
          </button>
        {/each}
        {#if $projectStore.projects.length === 0}
          <div class="text-sm text-[var(--color-text-muted)] px-3 py-2">No projects yet</div>
        {/if}
      </div>
    </div>

    <!-- New Project Button -->
    <div class="p-4 border-t border-[var(--color-surface)]">
      <button
        onclick={handleNewProject}
        class="w-full flex items-center justify-center gap-2 px-4 py-2 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 rounded-lg transition-colors font-medium"
      >
        <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
        New Project
      </button>
    </div>
  {:else}
    <!-- Collapsed state - just icons -->
    <div class="flex-1 flex flex-col items-center py-4 gap-2">
      <a
        href="/settings"
        class="p-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text)]"
        title="Settings"
      >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
        </svg>
      </a>
      <button
        onclick={handleNewProject}
        class="p-2 rounded-lg hover:bg-[var(--color-surface)] transition-colors text-[var(--color-text)]"
        title="New Project"
      >
        <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
        </svg>
      </button>
    </div>
  {/if}
</aside>
