import { describe, it, expect, beforeEach } from 'vitest';
import * as assets from '$lib/api/assets';
import { setBackend } from '$lib/api/assets';
import { MemoryBackend } from '$lib/api/memory-backend';
import type { AssetResponse } from '$lib/types/asset';

describe('assets API', () => {
  let backend: MemoryBackend;

  beforeEach(() => {
    backend = new MemoryBackend();
    setBackend(backend as import('$lib/api/storage-backend').StorageBackend);
  });

  const mockAsset: AssetResponse = {
    id: 'asset-123',
    project_id: 'proj-456',
    name: 'test-image.png',
    kind: 'Image',
    file_path: '/tmp/test-image.png',
    metadata: { width: 512, height: 512 },
    file_size: 102400,
    width: 512,
    height: 512,
    created_at: '2024-01-01T00:00:00Z',
  };

  describe('listAssets', () => {
    it('returns assets from backend', async () => {
      backend.on('list_assets', () => [mockAsset]);

      const result = await assets.listAssets('proj-456');

      expect(result).toEqual([mockAsset]);
    });

    it('passes projectId to backend', async () => {
      backend.on('list_assets', ({ projectId }: { projectId: string }) => {
        expect(projectId).toBe('proj-789');
        return [];
      });

      await assets.listAssets('proj-789');
    });
  });

  describe('getAsset', () => {
    it('calls correct command with id', async () => {
      backend.on('get_asset', ({ id }: { id: string }) => {
        expect(id).toBe('asset-123');
        return mockAsset;
      });

      const result = await assets.getAsset('asset-123');

      expect(result).toEqual(mockAsset);
    });
  });

  describe('deleteAsset', () => {
    it('calls correct command with id', async () => {
      let called = false;
      backend.on('delete_asset', ({ id }: { id: string }) => {
        expect(id).toBe('asset-123');
        called = true;
        return undefined;
      });

      await assets.deleteAsset('asset-123');

      expect(called).toBe(true);
    });
  });

  describe('importAsset', () => {
    it('calls correct command with request', async () => {
      backend.on('import_asset', ({ request }: { request: { project_id: string; source_path: string; name: string; kind: string } }) => {
        expect(request.project_id).toBe('proj-456');
        expect(request.source_path).toBe('/tmp/new-asset.png');
        expect(request.name).toBe('new-asset.png');
        expect(request.kind).toBe('Image');
        return mockAsset;
      });

      const result = await assets.importAsset({
        project_id: 'proj-456',
        source_path: '/tmp/new-asset.png',
        name: 'new-asset.png',
        kind: 'Image',
      });

      expect(result).toEqual(mockAsset);
    });
  });

  describe('generateImage', () => {
    it('calls correct command with request and returns job_id', async () => {
      backend.on('generate_image', ({ request }: { request: { project_id: string; prompt: string; width: number; height: number; steps: number } }) => {
        expect(request.project_id).toBe('proj-456');
        expect(request.prompt).toBe('A beautiful sunset');
        expect(request.width).toBe(512);
        expect(request.height).toBe(512);
        expect(request.steps).toBe(20);
        return 'job-789';
      });

      const result = await assets.generateImage({
        project_id: 'proj-456',
        prompt: 'A beautiful sunset',
        width: 512,
        height: 512,
        steps: 20,
      });

      expect(result).toBe('job-789');
    });

    it('includes optional fields in request', async () => {
      backend.on('generate_image', ({ request }: { request: { negative_prompt?: string; seed?: number; model_id?: string } }) => {
        expect(request.negative_prompt).toBe('blurry');
        expect(request.seed).toBe(42);
        expect(request.model_id).toBe('custom-model');
        return 'job-999';
      });

      const result = await assets.generateImage({
        project_id: 'proj-456',
        prompt: 'A beautiful sunset',
        negative_prompt: 'blurry',
        seed: 42,
        model_id: 'custom-model',
        width: 512,
        height: 512,
        steps: 20,
      });

      expect(result).toBe('job-999');
    });
  });
});
