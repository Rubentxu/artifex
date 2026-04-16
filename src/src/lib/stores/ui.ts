import { writable } from 'svelte/store';
import type { ProjectResponse } from '$lib/types';

// Lazy-initialized Tauri store to avoid import-time errors in tests
let store: Awaited<ReturnType<typeof import('@tauri-apps/plugin-store')['Store']['load']>> | null = null;
let hydrated = false;

export const sidebarCollapsed = writable<boolean>(false);
export const propertiesCollapsed = writable<boolean>(false);
export const selectedProjectId = writable<string | null>(null);
export const selectedProject = writable<ProjectResponse | null>(null);

export async function initStores(): Promise<void> {
  try {
    const { Store } = await import('@tauri-apps/plugin-store');
    store = new Store('panel-state.json');

    const savedSidebar = await store.get<boolean>('sidebarCollapsed');
    const savedProps = await store.get<boolean>('propertiesCollapsed');
    const savedProjId = await store.get<string | null>('selectedProjectId');

    if (savedSidebar !== undefined && savedSidebar !== null) {
      sidebarCollapsed.set(savedSidebar);
    }
    if (savedProps !== undefined && savedProps !== null) {
      propertiesCollapsed.set(savedProps);
    }
    if (savedProjId !== undefined && savedProjId !== null) {
      selectedProjectId.set(savedProjId);
    }
  } finally {
    hydrated = true;
  }
}

// Gate: only persist writes after hydration
if (typeof window !== 'undefined') {
  sidebarCollapsed.subscribe((value) => {
    if (hydrated && store) {
      store.set('sidebarCollapsed', value).catch(console.error);
    }
  });

  propertiesCollapsed.subscribe((value) => {
    if (hydrated && store) {
      store.set('propertiesCollapsed', value).catch(console.error);
    }
  });

  selectedProjectId.subscribe((value) => {
    if (hydrated && store) {
      store.set('selectedProjectId', value).catch(console.error);
    }
  });
}

// Helper to toggle sidebar
export function toggleSidebar(): void {
  sidebarCollapsed.update((v) => !v);
}

// Helper to toggle properties panel
export function toggleProperties(): void {
  propertiesCollapsed.update((v) => !v);
}

// Select a project
export function selectProject(project: ProjectResponse | null): void {
  selectedProject.set(project);
  selectedProjectId.set(project?.id ?? null);
}
