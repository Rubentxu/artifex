import { describe, it, expect, beforeEach } from 'vitest';
import { projectStore, selectedProject } from '../project';
import { setBackend } from '$lib/api/projects';
import { MemoryBackend } from '$lib/api/memory-backend';
import type { ProjectResponse } from '$lib/types';

describe('projectStore', () => {
  let backend: MemoryBackend;

  const mockProject: ProjectResponse = {
    id: '123e4567-e89b-12d3-a456-426614174000',
    name: 'TestGame',
    path: '/tmp/test-game',
    status: 'active',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  };

  const mockProject2: ProjectResponse = {
    id: '223e4567-e89b-12d3-a456-426614174001',
    name: 'AnotherGame',
    path: '/tmp/another-game',
    status: 'active',
    created_at: '2024-01-02T00:00:00Z',
    updated_at: '2024-01-02T00:00:00Z',
  };

  beforeEach(() => {
    backend = new MemoryBackend();
    setBackend(backend as import('$lib/api/storage-backend').StorageBackend);
    // Reset store state between tests
    projectStore.reset();
  });

  describe('loadProjects', () => {
    it('loads projects into store', async () => {
      backend.on('list_projects', () => [mockProject, mockProject2]);

      await projectStore.loadProjects();

      // Subscribe to get current value
      let state: typeof $projectStore;
      const unsubscribe = projectStore.subscribe(s => { state = s; });
      unsubscribe();

      expect(state?.projects).toHaveLength(2);
      expect(state?.projects[0].name).toBe('TestGame');
      expect(state?.loading).toBe(false);
    });

    it('sets error on failure', async () => {
      backend.on('list_projects', () => {
        throw new Error('Network error');
      });

      await projectStore.loadProjects();

      let state: typeof $projectStore;
      const unsubscribe = projectStore.subscribe(s => { state = s; });
      unsubscribe();

      expect(state?.error).toBe('Error: Network error');
      expect(state?.loading).toBe(false);
    });
  });

  describe('createProject', () => {
    it('adds new project to store', async () => {
      backend.on('list_projects', () => [mockProject]);
      backend.on('create_project', ({ request }: { request: { name: string; path: string } }) => ({
        ...mockProject,
        id: 'new-id',
        name: request.name,
        path: request.path,
      }));

      await projectStore.loadProjects();

      let beforeState: typeof $projectStore;
      projectStore.subscribe(s => { beforeState = s; })();

      const result = await projectStore.createProject('NewGame', '/tmp/new-game');

      expect(result.name).toBe('NewGame');

      let afterState: typeof $projectStore;
      projectStore.subscribe(s => { afterState = s; })();

      expect(beforeState?.projects).toHaveLength(1);
      expect(afterState?.projects).toHaveLength(2);
      expect(afterState?.projects[1].name).toBe('NewGame');
    });
  });

  describe('selectProject', () => {
    it('updates selectedId', async () => {
      backend.on('list_projects', () => [mockProject, mockProject2]);
      await projectStore.loadProjects();

      projectStore.selectProject(mockProject.id);

      let state: typeof $projectStore;
      const unsubscribe = projectStore.subscribe(s => { state = s; });
      unsubscribe();

      expect(state?.selectedId).toBe(mockProject.id);
    });

    it('can deselect by passing null', async () => {
      backend.on('list_projects', () => [mockProject]);
      await projectStore.loadProjects();

      projectStore.selectProject(mockProject.id);
      projectStore.selectProject(null);

      let state: typeof $projectStore;
      const unsubscribe = projectStore.subscribe(s => { state = s; });
      unsubscribe();

      expect(state?.selectedId).toBeNull();
    });
  });

  describe('renameProject', () => {
    it('updates project name in store', async () => {
      backend.on('list_projects', () => [mockProject]);
      backend.on('rename_project', ({ new_name }: { new_name: string }) => ({
        ...mockProject,
        name: new_name,
      }));
      await projectStore.loadProjects();

      const result = await projectStore.renameProject(mockProject.id, 'RenamedGame');

      expect(result.name).toBe('RenamedGame');

      let state: typeof $projectStore;
      const unsubscribe = projectStore.subscribe(s => { state = s; });
      unsubscribe();

      expect(state?.projects[0].name).toBe('RenamedGame');
    });
  });

  describe('archiveProject', () => {
    it('removes project from store', async () => {
      backend.on('list_projects', () => [mockProject, mockProject2]);
      backend.on('archive_project', () => undefined);
      await projectStore.loadProjects();

      await projectStore.archiveProject(mockProject.id);

      let state: typeof $projectStore;
      const unsubscribe = projectStore.subscribe(s => { state = s; });
      unsubscribe();

      expect(state?.projects).toHaveLength(1);
      expect(state?.projects[0].id).toBe(mockProject2.id);
    });

    it('clears selectedId if archived project was selected', async () => {
      backend.on('list_projects', () => [mockProject]);
      backend.on('archive_project', () => undefined);
      await projectStore.loadProjects();

      projectStore.selectProject(mockProject.id);
      await projectStore.archiveProject(mockProject.id);

      let state: typeof $projectStore;
      const unsubscribe = projectStore.subscribe(s => { state = s; });
      unsubscribe();

      expect(state?.selectedId).toBeNull();
    });
  });

  describe('selectedProject derived store', () => {
    it('returns the selected project object', async () => {
      backend.on('list_projects', () => [mockProject, mockProject2]);
      await projectStore.loadProjects();

      projectStore.selectProject(mockProject.id);

      let selected: ProjectResponse | null = null;
      const unsubscribe = selectedProject.subscribe(s => { selected = s; });
      unsubscribe();

      expect(selected?.name).toBe('TestGame');
    });

    it('returns null when nothing selected', async () => {
      backend.on('list_projects', () => [mockProject]);
      await projectStore.loadProjects();

      let selected: ProjectResponse | null = 'not-null';
      const unsubscribe = selectedProject.subscribe(s => { selected = s; });
      unsubscribe();

      expect(selected).toBeNull();
    });
  });
});
