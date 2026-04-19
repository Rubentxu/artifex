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

// Mock layer — only loaded in DEV mode (tree-shaken in production)
import {
  enableMock,
  disableMock,
  isMockMode,
  setMockData,
  setMockTier,
  setMockJobResult,
  simulateJobProgress,
  simulateJobCompleted,
  simulateJobFailed,
  getMockCalls,
  getMockCallHistory,
  resetMockCalls,
  getMockState,
} from './mock-layer';

const VERSION = '1.0.0';

let api: DebugAPI | null = null;

export function initDebugHarness(): void {
  // Subscribe to all stores
  initStoreInspector();

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
    // Mock layer API — only functional in DEV mode
    mock: import.meta.env.DEV
      ? {
          enableMock,
          disableMock,
          isMockMode,
          setMockData,
          setMockTier,
          setMockJobResult,
          simulateJobProgress,
          simulateJobCompleted,
          simulateJobFailed,
          getMockCalls,
          getMockCallHistory,
          resetMockCalls,
          getMockState,
        }
      : undefined,
  };

  // Attach to global
  (window as unknown as Record<string, unknown>).__ARTIFEX_DEBUG__ = api;

  console.log('[debug-harness] initialized', VERSION);
}

export function destroyDebugHarness(): void {
  if (api) {
    destroyStoreInspector();
    // Ensure mock is disabled on destroy
    if (import.meta.env.DEV && api.mock?.isMockMode()) {
      api.mock.disableMock();
    }
    delete (window as unknown as Record<string, unknown>).__ARTIFEX_DEBUG__;
    api = null;
    console.log('[debug-harness] destroyed');
  }
}
