import { writable, derived } from 'svelte/store';
import type { ProjectResponse } from '$lib/types';
import * as projectApi from '$lib/api/projects';

interface ProjectState {
  projects: ProjectResponse[];
  selectedId: string | null;
  loading: boolean;
  error: string | null;
}

function createProjectStore() {
  const { subscribe, set, update } = writable<ProjectState>({
    projects: [],
    selectedId: null,
    loading: false,
    error: null,
  });

  return {
    subscribe,

    reset() {
      set({ projects: [], selectedId: null, loading: false, error: null });
    },

    async loadProjects() {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const projects = await projectApi.listProjects();
        update(s => ({ ...s, projects, loading: false }));
      } catch (e) {
        update(s => ({ ...s, error: String(e), loading: false }));
      }
    },

    async createProject(name: string, path: string) {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const project = await projectApi.createProject(name, path);
        update(s => ({
          ...s,
          projects: [...s.projects, project],
          loading: false,
        }));
        return project;
      } catch (e) {
        update(s => ({ ...s, error: String(e), loading: false }));
        throw e;
      }
    },

    selectProject(id: string | null) {
      update(s => ({ ...s, selectedId: id }));
    },

    async renameProject(id: string, newName: string) {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const project = await projectApi.renameProject(id, newName);
        update(s => ({
          ...s,
          projects: s.projects.map(p => p.id === id ? project : p),
          loading: false,
        }));
        return project;
      } catch (e) {
        update(s => ({ ...s, error: String(e), loading: false }));
        throw e;
      }
    },

    async archiveProject(id: string) {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        await projectApi.archiveProject(id);
        update(s => ({
          ...s,
          projects: s.projects.filter(p => p.id !== id),
          selectedId: s.selectedId === id ? null : s.selectedId,
          loading: false,
        }));
      } catch (e) {
        update(s => ({ ...s, error: String(e), loading: false }));
        throw e;
      }
    },
  };
}

export const projectStore = createProjectStore();

export const selectedProject = derived(projectStore, ($state) =>
  $state.projects.find(p => p.id === $state.selectedId) ?? null
);
