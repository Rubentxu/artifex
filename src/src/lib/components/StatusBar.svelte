<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { selectedProject } from '$lib/stores/project';
  import { listRoutingRules, listModelProfiles, type RoutingRuleDto, type ModelProfileDto } from '$lib/api/model-config';

  const version = '0.1.0';

  interface JobProgress {
    job_id: string;
    progress_percent: number;
    progress_message: string;
  }

  interface JobCompleted {
    job_id: string;
    asset_ids: string[];
  }

  interface JobFailed {
    job_id: string;
    error_message: string;
  }

  let activeJobs = $state(0);
  let currentProgress = $state<JobProgress | null>(null);
  let statusMessage = $state<string | null>(null);
  let statusTimeout: ReturnType<typeof setTimeout> | null = null;
  let unlistenFns: (() => void)[] = [];

  // Active model info
  let activeModel = $state<string | null>(null);
  let activeProvider = $state<string | null>(null);

  function showStatus(message: string, duration = 5000) {
    statusMessage = message;
    if (statusTimeout) clearTimeout(statusTimeout);
    if (duration > 0) {
      statusTimeout = setTimeout(() => {
        statusMessage = null;
      }, duration);
    }
  }

  async function loadActiveModel() {
    try {
      const [rules, profiles] = await Promise.all([
        listRoutingRules(),
        listModelProfiles(),
      ]);

      // Find the active model for image generation (primary operation)
      const imageRule = rules.find(r => r.operation_type === 'imagegen.txt2img');
      if (imageRule) {
        const profile = profiles.find(p => p.id === imageRule.default_profile_id);
        if (profile) {
          activeModel = profile.display_name;
          activeProvider = profile.provider_name;
        } else {
          // Rule references a profile that doesn't exist
          activeModel = 'Invalid profile';
          activeProvider = null;
        }
      } else {
        // No routing rule configured
        activeModel = 'No model configured';
        activeProvider = null;
      }
    } catch (e) {
      console.warn('Failed to load active model:', e);
      activeModel = 'No model configured';
      activeProvider = null;
    }
  }

  onMount(async () => {
    try {
        const { listen } = await import('@tauri-apps/api/event');

        const unlistenProgress = await listen<JobProgress>('job-progress', (event) => {
          activeJobs = 1;
          currentProgress = event.payload;
        });
        unlistenFns.push(unlistenProgress);

        const unlistenCompleted = await listen<JobCompleted>('job-completed', (event) => {
          activeJobs = Math.max(0, activeJobs - 1);
          currentProgress = null;
          if (activeJobs === 0) {
            // Determine asset type from number of asset_ids (currently all jobs produce 1 asset)
            const assetCount = event.payload.asset_ids.length;
            if (assetCount > 0) {
              showStatus('✓ Asset generated');
            } else {
              showStatus('✓ Job completed');
            }
          }
        });
        unlistenFns.push(unlistenCompleted);

        const unlistenFailed = await listen<JobFailed>('job-failed', (event) => {
          activeJobs = Math.max(0, activeJobs - 1);
          currentProgress = null;
          showStatus('✗ Generation failed: ' + event.payload.error_message);
        });
        unlistenFns.push(unlistenFailed);
      } catch (e) {
        console.warn('Failed to listen to Tauri events:', e);
      }

    // Load active model info
    await loadActiveModel();
  });

  onDestroy(() => {
    unlistenFns.forEach(fn => fn());
    if (statusTimeout) clearTimeout(statusTimeout);
  });
</script>

<footer class="h-8 bg-[var(--color-panel)] border-t border-[var(--color-surface)] flex items-center justify-between px-4 text-xs text-[var(--color-text-muted)]">
  <!-- Left: App version and active model -->
  <div class="flex items-center gap-3">
    <span>Artifex v{version}</span>
    {#if activeModel && activeProvider}
      <span class="text-[var(--color-accent)]">•</span>
      <span class="truncate max-w-xs" title="{activeModel} ({activeProvider})">
        {activeModel} ({activeProvider})
      </span>
    {/if}
  </div>

  <!-- Center: Current project -->
  <div class="flex items-center gap-2">
    {#if $selectedProject}
      <span class="truncate max-w-md">{$selectedProject.name}</span>
    {:else}
      <span>No project selected</span>
    {/if}
  </div>

  <!-- Right: Job status -->
  <div class="flex items-center gap-2">
    {#if statusMessage}
      <span>{statusMessage}</span>
    {:else if currentProgress}
      <span>Processing... {currentProgress.progress_percent}% {currentProgress.progress_message}</span>
    {:else if activeJobs > 0}
      <span>Processing... ({activeJobs} job{activeJobs > 1 ? 's' : ''})</span>
    {:else}
      <span class="opacity-50">Ready</span>
    {/if}
  </div>
</footer>
