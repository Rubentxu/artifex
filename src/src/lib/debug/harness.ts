// Main debug harness entry point — wires store-inspector + dom-inspector + route/viewport/tauri
import { initStoreInspector, getStores, getStore, destroyStoreInspector } from './store-inspector';
import {
  getElement,
  getElements,
  getViewport,
  getRoute,
  getTauriContext,
  getNavigation,
  getDialogs,
  getButtons,
  getTextContent,
  hasText,
  getActiveJobs,
  waitForCondition,
} from './dom-inspector';
import type { DebugAPI } from './types';

const VERSION = '1.0.0';

let api: DebugAPI | null = null;

export function initDebugHarness(): void {
  // Subscribe to all stores
  initStoreInspector();

  // @ts-ignore — __ARTIFEX_E2E__ is injected by vite.config.ts define
  const isE2E = __ARTIFEX_E2E__;

  // Build the API object
  api = {
    getStores,
    getStore,
    getRoute,
    getElement,
    getElements,
    getNavigation,
    getDialogs,
    getButtons,
    getTextContent,
    hasText,
    getViewport,
    getActiveJobs,
    getTauriContext,
    waitForCondition,
    snapshot() {
      return {
        stores: getStores(),
        route: getRoute(),
        viewport: getViewport(),
        tauri: getTauriContext(),
        timestamp: Date.now(),
      };
    },
    destroy() {
      destroyDebugHarness();
    },
    version: VERSION,
    // Mock layer API — lazily loaded only in E2E mode
    mock: isE2E ? {
      enableMock: async () => {
        const mock = await import('./mock-layer');
        return mock.enableMock();
      },
      disableMock: async () => {
        const mock = await import('./mock-layer');
        return mock.disableMock();
      },
      isMockMode: async () => {
        const mock = await import('./mock-layer');
        return mock.isMockMode();
      },
      setMockData: async (data: unknown) => {
        const mock = await import('./mock-layer');
        return mock.setMockData(data);
      },
      setMockTier: async (tier: string) => {
        const mock = await import('./mock-layer');
        return mock.setMockTier(tier);
      },
      setMockJobResult: async (result: unknown) => {
        const mock = await import('./mock-layer');
        return mock.setMockJobResult(result);
      },
      simulateJobProgress: async (jobId: string, progress: number) => {
        const mock = await import('./mock-layer');
        return mock.simulateJobProgress(jobId, progress);
      },
      simulateJobCompleted: async (jobId: string) => {
        const mock = await import('./mock-layer');
        return mock.simulateJobCompleted(jobId);
      },
      simulateJobFailed: async (jobId: string, error: string) => {
        const mock = await import('./mock-layer');
        return mock.simulateJobFailed(jobId, error);
      },
      getMockCalls: async () => {
        const mock = await import('./mock-layer');
        return mock.getMockCalls();
      },
      getMockCallHistory: async () => {
        const mock = await import('./mock-layer');
        return mock.getMockCallHistory();
      },
      resetMockCalls: async () => {
        const mock = await import('./mock-layer');
        return mock.resetMockCalls();
      },
      getMockState: async () => {
        const mock = await import('./mock-layer');
        return mock.getMockState();
      },
    } : undefined,
  };

  // Attach to global
  (window as unknown as Record<string, unknown>).__ARTIFEX_DEBUG__ = api;

  console.log('[debug-harness] initialized', VERSION);
}

export function destroyDebugHarness(): void {
  if (api) {
    destroyStoreInspector();
    delete (window as unknown as Record<string, unknown>).__ARTIFEX_DEBUG__;
    api = null;
    console.log('[debug-harness] destroyed');
  }
}
