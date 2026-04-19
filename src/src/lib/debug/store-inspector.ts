// Store subscription + snapshot caching for the debug harness
import { projectStore, selectedProject } from '$lib/stores/project';
import { assetStore, selectedAsset, filteredAssets } from '$lib/stores/asset';
import { identityStore, currentTier, currentUser, isPro } from '$lib/stores/identity';
import { agentStore, isAgentRunning, currentPhaseInfo } from '$lib/stores/agent';
import {
  sidebarCollapsed,
  propertiesCollapsed,
  selectedProjectId,
  selectedProject as uiSelectedProject,
} from '$lib/stores/ui';

type Subscribable = { subscribe: (fn: (v: unknown) => void) => () => void };

interface StoreEntry {
  value: unknown;
  unsubscribe: () => void;
}

// Internal registry: name → { value, unsubscribe }
const registry = new Map<string, StoreEntry>();

function subscribeToStore(name: string, store: Subscribable): void {
  // Capture initial value
  let current: unknown;
  const unsub = store.subscribe((v) => {
    current = v;
  });
  // Store initial snapshot
  registry.set(name, { value: current, unsubscribe: unsub });
}

export function initStoreInspector(): void {
  // UI stores (plain writables)
  subscribeToStore('ui.sidebarCollapsed', sidebarCollapsed);
  subscribeToStore('ui.propertiesCollapsed', propertiesCollapsed);
  subscribeToStore('ui.selectedProjectId', selectedProjectId);
  subscribeToStore('ui.selectedProject', uiSelectedProject);
  // Wrapped stores
  subscribeToStore('project', projectStore);
  subscribeToStore('project.selectedProject', selectedProject);
  subscribeToStore('asset', assetStore);
  subscribeToStore('asset.selectedAsset', selectedAsset);
  subscribeToStore('asset.filteredAssets', filteredAssets);
  subscribeToStore('identity', identityStore);
  subscribeToStore('identity.currentTier', currentTier);
  subscribeToStore('identity.currentUser', currentUser);
  subscribeToStore('identity.isPro', isPro);
  subscribeToStore('agent', agentStore);
  subscribeToStore('agent.isAgentRunning', isAgentRunning);
  subscribeToStore('agent.currentPhaseInfo', currentPhaseInfo);
}

export function getStores(): Record<string, unknown> {
  const result: Record<string, unknown> = {};
  for (const [name, entry] of registry) {
    result[name] = entry.value;
  }
  return result;
}

export function getStore(name: string): unknown {
  return registry.get(name)?.value;
}

export function destroyStoreInspector(): void {
  for (const entry of registry.values()) {
    entry.unsubscribe();
  }
  registry.clear();
}
