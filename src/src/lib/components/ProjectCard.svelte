<script lang="ts">
  import type { ProjectResponse } from '$lib/types';
  import { selectedProject, selectProject } from '$lib/stores/ui';
  import Badge from '$lib/components/ui/Badge.svelte';

  interface Props {
    project: ProjectResponse;
    onContextMenu?: (event: MouseEvent, project: ProjectResponse) => void;
  }

  let { project, onContextMenu }: Props = $props();

  function formatDate(dateStr: string): string {
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString('en-US', {
        year: 'numeric',
        month: 'short',
        day: 'numeric',
      });
    } catch {
      return dateStr;
    }
  }

  function handleClick() {
    selectProject(project);
  }

  function handleContextMenu(event: MouseEvent) {
    event.preventDefault();
    onContextMenu?.(event, project);
  }
</script>

<button
  onclick={handleClick}
  oncontextmenu={handleContextMenu}
  class="w-full text-left p-4 rounded-sm border transition-all duration-150 hover:border-[var(--color-neon)]/50 hover:shadow-[var(--glow-neon)] bg-[var(--color-panel)] hover:bg-[var(--color-panel)]/80 border-[var(--color-surface)]"
  class:ring-2={$selectedProject?.id === project.id}
  class:ring-[var(--color-neon)]={$selectedProject?.id === project.id}
>
  <!-- Header: Name and Status -->
  <div class="flex items-start justify-between gap-2 mb-2">
    <h3 class="font-semibold text-lg truncate">{project.name}</h3>
    <Badge status={project.status}>{project.status}</Badge>
  </div>

  <!-- Path -->
  <p class="text-sm text-[var(--color-text-muted)] truncate mb-2">{project.path}</p>

  <!-- Footer: Date -->
  <div class="flex items-center justify-between text-xs text-[var(--color-text-muted)]">
    <span>Created {formatDate(project.created_at)}</span>
  </div>
</button>
