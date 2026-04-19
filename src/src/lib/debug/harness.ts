// Main debug harness entry point — wires store-inspector + dom-inspector + route/viewport/tauri
import { initStoreInspector, getStores, getStore, destroyStoreInspector } from './store-inspector';
import { getElement, getElements, getViewport, getRoute, getTauriContext } from './dom-inspector';
import type { DebugAPI } from './types';

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
    getViewport,
    getTauriContext,
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
