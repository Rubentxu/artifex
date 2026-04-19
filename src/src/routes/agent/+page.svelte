<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { agentStore, isAgentRunning } from '$lib/stores/agent';
  import { selectedProject } from '$lib/stores/project';
  import { startCodeAgent } from '$lib/api/agent';
  import type { AgentStepProgress } from '$lib/types';

  let prompt = $state('');
  let engine = $state<'godot' | 'unity'>('godot');
  let unlistenProgress: (() => void) | null = null;

  // Subscribe to agent store
  let agentState = $state({ messages: [] as { role: string; content: string }[], currentPhase: null as string | null, stepName: null as string | null, percent: 0, jobId: null as string | null, loading: false, error: null as string | null });

  const unsubscribe = agentStore.subscribe(state => {
    agentState = state;
  });

  onMount(async () => {
    // Listen for job-progress events
    try {
      const { listen } = await import('@tauri-apps/api/event');
      unlistenProgress = await listen<AgentStepProgress>('job-progress', (event) => {
        agentStore.updateProgress(event.payload);
      });
    } catch (e) {
      console.warn('Failed to listen for job-progress event:', e);
    }
  });

  onDestroy(() => {
    unsubscribe();
    if (unlistenProgress) {
      unlistenProgress();
    }
  });

  async function handleSubmit() {
    if (!$selectedProject) return;
    if (prompt.trim().isEmpty()) return;

    agentStore.setLoading(true);
    agentStore.addMessage('user', prompt);

    try {
      const jobId = await startCodeAgent({
        projectId: $selectedProject.id,
        engine,
        prompt: prompt.trim(),
      });
      agentStore.setJobId(jobId);
      prompt = '';
    } catch (e) {
      agentStore.setError(e instanceof Error ? e.message : String(e));
      agentStore.addMessage('assistant', `Error: ${e instanceof Error ? e.message : String(e)}`);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  function getPhaseLabel(phase: string | null): string {
    switch (phase) {
      case 'plan': return 'Planning';
      case 'execute': return 'Executing';
      case 'verify': return 'Verifying';
      case 'refine': return 'Refining';
      case 'done': return 'Done';
      case 'failed': return 'Failed';
      default: return 'Idle';
    }
  }
</script>

<div class="h-full flex flex-col overflow-hidden">
  <!-- Header -->
  <header class="flex items-center justify-between px-6 py-4 border-b border-[var(--color-surface)]">
    <h1 class="text-2xl font-bold">Code Agent</h1>
    <div class="flex items-center gap-4">
      <select
        bind:value={engine}
        class="px-3 py-2 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)]"
        disabled={agentState.loading}
      >
        <option value="godot">Godot</option>
        <option value="unity">Unity</option>
      </select>
    </div>
  </header>

  <!-- Progress Bar -->
  {#if agentState.loading && agentState.currentPhase}
    <div class="px-6 py-3 bg-[var(--color-surface)] border-b border-[var(--color-border)]">
      <div class="flex items-center justify-between mb-2">
        <span class="text-sm font-medium">{getPhaseLabel(agentState.currentPhase)}</span>
        <span class="text-sm text-[var(--color-text-muted)]">{agentState.stepName || ''}</span>
      </div>
      <div class="w-full h-2 bg-[var(--color-panel)] rounded-full overflow-hidden">
        <div
          class="h-full bg-[var(--color-accent)] transition-all duration-300"
          style="width: {agentState.percent}%"
        ></div>
      </div>
    </div>
  {/if}

  <!-- Error Display -->
  {#if agentState.error}
    <div class="mx-6 mt-3 p-3 rounded-lg bg-red-500/20 border border-red-500/50 text-red-400 text-sm">
      {agentState.error}
    </div>
  {/if}

  <!-- Conversation -->
  <main class="flex-1 overflow-y-auto p-6">
    {#if agentState.messages.length === 0}
      <div class="flex flex-col items-center justify-center h-full text-center">
        <div class="w-24 h-24 rounded-full bg-[var(--color-panel)] flex items-center justify-center mb-4">
          <svg class="w-12 h-12 text-[var(--color-text-muted)]" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.75 17L9 20l-1 1h8l-1-1-.75-3M3 13h18M5 17h14a2 2 0 002-2V5a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
          </svg>
        </div>
        <h2 class="text-xl font-semibold mb-2">Code Agent</h2>
        <p class="text-[var(--color-text-muted)] mb-4">Describe what you want to build and the agent will create it step by step</p>
      </div>
    {:else}
      <div class="space-y-4">
        {#each agentState.messages as message, i}
          <div class="flex {message.role === 'user' ? 'justify-end' : 'justify-start'}">
            <div class="max-w-[70%] rounded-lg p-4 {message.role === 'user' ? 'bg-[var(--color-accent)] text-white' : 'bg-[var(--color-surface)]'}">
              <div class="text-xs opacity-70 mb-1 capitalize">{message.role}</div>
              <div class="prose prose-sm max-w-none whitespace-pre-wrap">{message.content}</div>
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </main>

  <!-- Input -->
  <footer class="p-6 border-t border-[var(--color-surface)]">
    <div class="flex gap-4">
      <textarea
        bind:value={prompt}
        onkeydown={handleKeydown}
        placeholder="Describe what you want to build..."
        class="flex-1 min-h-[80px] px-4 py-3 rounded-lg bg-[var(--color-surface)] border border-[var(--color-border)] resize-none"
        disabled={agentState.loading || !$selectedProject}
      ></textarea>
      <button
        onclick={handleSubmit}
        disabled={agentState.loading || !$selectedProject || prompt.trim().isEmpty()}
        class="px-6 py-3 bg-[var(--color-accent)] hover:bg-[var(--color-accent)]/80 disabled:bg-[var(--color-surface)] disabled:cursor-not-allowed rounded-lg font-medium transition-colors"
      >
        {agentState.loading ? 'Running...' : 'Send'}
      </button>
    </div>
    {#if !$selectedProject}
      <p class="mt-2 text-sm text-[var(--color-text-muted)]">Select a project first</p>
    {/if}
  </footer>
</div>
