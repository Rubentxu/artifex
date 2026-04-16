import { describe, it, expect, beforeEach } from 'vitest';
import * as projects from '$lib/api/projects';
import { setBackend } from '$lib/api/projects';
import { MemoryBackend } from '$lib/api/memory-backend';
import type { ProjectResponse } from '$lib/types';

describe('projects API', () => {
  let backend: MemoryBackend;

  beforeEach(() => {
    backend = new MemoryBackend();
    setBackend(backend as import('$lib/api/storage-backend').StorageBackend);
  });

  const mockProject: ProjectResponse = {
    id: '123e4567-e89b-12d3-a456-426614174000',
    name: 'TestGame',
    path: '/tmp/test-game',
    status: 'active',
    created_at: '2024-01-01T00:00:00Z',
    updated_at: '2024-01-01T00:00:00Z',
  };

  describe('listProjects', () => {
    it('returns projects from backend', async () => {
      backend.on('list_projects', () => [mockProject]);

      const result = await projects.listProjects();

      expect(result).toEqual([mockProject]);
    });

    it('passes includeArchived to backend', async () => {
      backend.on('list_projects', ({ includeArchived }: { includeArchived: boolean }) => {
        expect(includeArchived).toBe(true);
        return [];
      });

      await projects.listProjects(true);
    });
  });

  describe('createProject', () => {
    it('calls correct command with name and path', async () => {
      backend.on('create_project', ({ request }: { request: { name: string; path: string } }) => {
        expect(request.name).toBe('NewGame');
        expect(request.path).toBe('/tmp/new-game');
        return { ...mockProject, name: 'NewGame' };
      });

      const result = await projects.createProject('NewGame', '/tmp/new-game');

      expect(result.name).toBe('NewGame');
    });
  });

  describe('getProject', () => {
    it('calls correct command with id', async () => {
      backend.on('get_project', ({ id }: { id: string }) => {
        expect(id).toBe('123e4567-e89b-12d3-a456-426614174000');
        return mockProject;
      });

      const result = await projects.getProject('123e4567-e89b-12d3-a456-426614174000');

      expect(result).toEqual(mockProject);
    });
  });

  describe('openProject', () => {
    it('calls correct command with id', async () => {
      backend.on('open_project', ({ id }: { id: string }) => {
        expect(id).toBe('123e4567-e89b-12d3-a456-426614174000');
        return mockProject;
      });

      const result = await projects.openProject('123e4567-e89b-12d3-a456-426614174000');

      expect(result).toEqual(mockProject);
    });
  });

  describe('renameProject', () => {
    it('calls correct command with id and new_name', async () => {
      backend.on('rename_project', ({ id, new_name }: { id: string; new_name: string }) => {
        expect(id).toBe('123e4567-e89b-12d3-a456-426614174000');
        expect(new_name).toBe('RenamedGame');
        return { ...mockProject, name: 'RenamedGame' };
      });

      const result = await projects.renameProject('123e4567-e89b-12d3-a456-426614174000', 'RenamedGame');

      expect(result.name).toBe('RenamedGame');
    });
  });

  describe('archiveProject', () => {
    it('calls correct command with id', async () => {
      let called = false;
      backend.on('archive_project', ({ id }: { id: string }) => {
        expect(id).toBe('123e4567-e89b-12d3-a456-426614174000');
        called = true;
        return undefined;
      });

      await projects.archiveProject('123e4567-e89b-12d3-a456-426614174000');

      expect(called).toBe(true);
    });
  });

  describe('deleteProject', () => {
    it('calls correct command with id', async () => {
      let called = false;
      backend.on('delete_project', ({ id }: { id: string }) => {
        expect(id).toBe('123e4567-e89b-12d3-a456-426614174000');
        called = true;
        return undefined;
      });

      await projects.deleteProject('123e4567-e89b-12d3-a456-426614174000');

      expect(called).toBe(true);
    });
  });
});
