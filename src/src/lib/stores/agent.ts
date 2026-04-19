import { writable, derived } from 'svelte/store';
import type { AgentStepProgress } from '$lib/types';

export interface AgentMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
}

interface AgentState {
  messages: AgentMessage[];
  currentPhase: string | null;
  stepName: string | null;
  percent: number;
  jobId: string | null;
  loading: boolean;
  error: string | null;
}

function createAgentStore() {
  const { subscribe, set, update } = writable<AgentState>({
    messages: [],
    currentPhase: null,
    stepName: null,
    percent: 0,
    jobId: null,
    loading: false,
    error: null,
  });

  return {
    subscribe,

    setLoading(loading: boolean) {
      update(s => ({ ...s, loading, error: loading ? null : s.error }));
    },

    setError(error: string | null) {
      update(s => ({ ...s, error, loading: false }));
    },

    setJobId(jobId: string | null) {
      update(s => ({ ...s, jobId }));
    },

    updateProgress(progress: AgentStepProgress) {
      update(s => ({
        ...s,
        currentPhase: progress.phase,
        stepName: progress.stepName,
        percent: progress.percent,
      }));
    },

    addMessage(role: AgentMessage['role'], content: string) {
      update(s => ({
        ...s,
        messages: [...s.messages, { role, content }],
      }));
    },

    setMessages(messages: AgentMessage[]) {
      update(s => ({ ...s, messages }));
    },

    reset() {
      set({
        messages: [],
        currentPhase: null,
        stepName: null,
        percent: 0,
        jobId: null,
        loading: false,
        error: null,
      });
    },
  };
}

export const agentStore = createAgentStore();

export const isAgentRunning = derived(
  agentStore,
  ($state) => $state.loading && $state.jobId !== null
);

export const currentPhaseInfo = derived(agentStore, ($state) => ({
  phase: $state.currentPhase,
  stepName: $state.stepName,
  percent: $state.percent,
}));
