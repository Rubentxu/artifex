<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { listJobs, type ListJobsFilters } from '$lib/api/jobs';
  import { assetStore } from '$lib/stores/asset';
  import type { JobResponse } from '$lib/types';

  interface Props {
    projectId: string;
  }

  let { projectId }: Props = $props();

  type StatusFilter = 'all' | 'running' | 'completed' | 'failed';
  let statusFilter = $state<StatusFilter>('all');
  let jobs = $state<JobResponse[]>([]);
  let loading = $state(false);
  let unlistenFns: (() => void)[] = [];

  async function loadJobs() {
    loading = true;
    try {
      const filters: ListJobsFilters = { limit: 20 };
      if (statusFilter !== 'all') {
        filters.status = statusFilter;
      }
      jobs = await listJobs(projectId, filters);
    } catch (e) {
      console.warn('Failed to load jobs:', e);
    } finally {
      loading = false;
    }
  }

  async function handleRetry(job: JobResponse) {
    try {
      switch (job.job_type) {
        case 'image_generate':
          await assetStore.generateImage(job.operation as Parameters<typeof assetStore.generateImage>[0]);
          break;
        case 'audio_generate':
          await assetStore.generateAudio(job.operation as Parameters<typeof assetStore.generateAudio>[0]);
          break;
        case 'tts_synthesize':
          await assetStore.synthesizeSpeech(job.operation as Parameters<typeof assetStore.synthesizeSpeech>[0]);
          break;
        case 'image_remove_background':
          await assetStore.removeBackground(job.operation as Parameters<typeof assetStore.removeBackground>[0]);
          break;
        case 'pixel_art_convert':
          await assetStore.convertPixelArt(job.operation as Parameters<typeof assetStore.convertPixelArt>[0]);
          break;
        case 'tile_generate':
          await assetStore.generateTile(job.operation as Parameters<typeof assetStore.generateTile>[0]);
          break;
        default:
          console.error(`Unknown job type for retry: ${job.job_type}`);
      }
    } catch (e) {
      console.error('Retry failed:', e);
    }
  }

  function formatTimestamp(ts: string | null): string {
    if (!ts) return '—';
    try {
      return new Date(ts).toLocaleString();
    } catch {
      return ts;
    }
  }

  function jobTypeIcon(jobType: string): string {
    switch (jobType) {
      case 'image_generate': return '🖼';
      case 'audio_generate': return '🔊';
      case 'tts_synthesize': return '🎤';
      default: return '⚙';
    }
  }

  function statusBadge(status: string): { cls: string; label: string } {
    switch (status.toLowerCase()) {
      case 'running': return { cls: 'bg-blue-500/20 text-blue-400', label: 'Running' };
      case 'completed': return { cls: 'bg-green-500/20 text-green-400', label: 'Completed' };
      case 'failed': return { cls: 'bg-red-500/20 text-red-400', label: 'Failed' };
      case 'pending': return { cls: 'bg-yellow-500/20 text-yellow-400', label: 'Pending' };
      default: return { cls: 'bg-gray-500/20 text-gray-400', label: status };
    }
  }

  $effect(() => {
    // Reload when status filter changes
    statusFilter;
    loadJobs();
  });

  onMount(async () => {
    await loadJobs();

    try {
      const { listen } = await import('@tauri-apps/api/event');
      const unlistenCompleted = await listen('job-completed', () => {
        loadJobs();
      });
      const unlistenFailed = await listen('job-failed', () => {
        loadJobs();
      });
      unlistenFns.push(unlistenCompleted, unlistenFailed);
    } catch (e) {
      console.warn('Failed to listen to job events:', e);
    }
  });

  onDestroy(() => {
    unlistenFns.forEach(fn => fn());
  });
</script>

<section class="job-history border-t border-[var(--color-surface)] mt-6">
  <div class="flex items-center justify-between px-6 py-3">
    <h3 class="text-lg font-semibold">Recent Jobs</h3>
    <div class="flex gap-1">
      {#each (['all', 'running', 'completed', 'failed'] as StatusFilter[]) as filter}
        <button
          onclick={() => (statusFilter = filter)}
          class="px-3 py-1 rounded text-xs font-medium transition-colors {statusFilter === filter
            ? 'bg-[var(--color-accent)] text-white'
            : 'bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 text-[var(--color-text-muted)]'}"
        >
          {filter.charAt(0).toUpperCase() + filter.slice(1)}
        </button>
      {/each}
    </div>
  </div>

  <div class="px-6 pb-4">
    {#if loading}
      <div class="flex items-center justify-center py-8 text-[var(--color-text-muted)]">
        <span class="text-sm">Loading jobs...</span>
      </div>
    {:else if jobs.length === 0}
      <div class="flex flex-col items-center justify-center py-8 text-[var(--color-text-muted)]">
        <span class="text-sm">No jobs found</span>
      </div>
    {:else}
      <div class="space-y-2">
        {#each jobs as job (job.id)}
          {@const badge = statusBadge(job.status)}
          <div class="flex items-center gap-3 p-3 rounded-lg bg-[var(--color-panel)] border border-[var(--color-surface)]">
            <span class="text-lg">{jobTypeIcon(job.job_type)}</span>
            <div class="flex-1 min-w-0">
              <div class="flex items-center gap-2">
                <span class="text-sm font-medium truncate">{job.job_type}</span>
                <span class="shrink-0 px-2 py-0.5 rounded text-xs {badge.cls}">{badge.label}</span>
              </div>
              <div class="flex items-center gap-3 text-xs text-[var(--color-text-muted)] mt-0.5">
                <span>Created: {formatTimestamp(job.created_at)}</span>
                {#if job.started_at}
                  <span>Started: {formatTimestamp(job.started_at)}</span>
                {/if}
                {#if job.completed_at}
                  <span>Completed: {formatTimestamp(job.completed_at)}</span>
                {/if}
                {#if job.progress_percent > 0 && job.status === 'running'}
                  <span>{job.progress_percent}% — {job.progress_message ?? ''}</span>
                {/if}
              </div>
              {#if job.error_message}
                <div class="text-xs text-red-400 mt-1 truncate">{job.error_message}</div>
              {/if}
            </div>
            {#if job.status.toLowerCase() === 'failed'}
              <button
                onclick={() => handleRetry(job)}
                class="shrink-0 px-3 py-1.5 rounded text-xs font-medium bg-[var(--color-surface)] hover:bg-[var(--color-surface)]/80 transition-colors"
              >
                Retry
              </button>
            {/if}
          </div>
        {/each}
      </div>
    {/if}
  </div>
</section>
