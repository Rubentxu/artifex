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

  describe('removeBackground', () => {
    it('calls remove_background with correct params', async () => {
      backend.on('remove_background', ({ request }: { request: { project_id: string; asset_id: string; provider_mode?: string } }) => {
        expect(request.project_id).toBe('proj-456');
        expect(request.asset_id).toBe('asset-123');
        expect(request.provider_mode).toBeUndefined();
        return 'job-rb-001';
      });

      const result = await assets.removeBackground({
        project_id: 'proj-456',
        asset_id: 'asset-123',
      });

      expect(result).toBe('job-rb-001');
    });

    it('includes optional provider_mode', async () => {
      backend.on('remove_background', ({ request }: { request: { project_id: string; asset_id: string; provider_mode?: string } }) => {
        expect(request.provider_mode).toBe('kie');
        return 'job-rb-002';
      });

      const result = await assets.removeBackground({
        project_id: 'proj-456',
        asset_id: 'asset-123',
        provider_mode: 'kie',
      });

      expect(result).toBe('job-rb-002');
    });
  });

  describe('convertPixelArt', () => {
    it('calls convert_pixel_art with correct params', async () => {
      backend.on('convert_pixel_art', ({ request }: { request: { project_id: string; asset_id: string; target_width: number; target_height: number; palette: { type: string }; dithering: string; outline: boolean; outline_threshold?: number } }) => {
        expect(request.project_id).toBe('proj-456');
        expect(request.asset_id).toBe('asset-123');
        expect(request.target_width).toBe(64);
        expect(request.target_height).toBe(64);
        expect(request.palette).toEqual({ type: 'Pico8' });
        expect(request.dithering).toBe('floyd_steinberg');
        expect(request.outline).toBe(true);
        return 'job-pa-001';
      });

      const result = await assets.convertPixelArt({
        project_id: 'proj-456',
        asset_id: 'asset-123',
        target_width: 64,
        target_height: 64,
        palette: { type: 'Pico8' },
        dithering: 'floyd_steinberg',
        outline: true,
      });

      expect(result).toBe('job-pa-001');
    });

    it('includes optional outline_threshold', async () => {
      backend.on('convert_pixel_art', ({ request }: { request: { project_id: string; asset_id: string; target_width: number; target_height: number; palette: { type: string }; dithering: string; outline: boolean; outline_threshold?: number } }) => {
        expect(request.outline_threshold).toBe(128);
        return 'job-pa-002';
      });

      const result = await assets.convertPixelArt({
        project_id: 'proj-456',
        asset_id: 'asset-123',
        target_width: 64,
        target_height: 64,
        palette: { type: 'GameBoy' },
        dithering: 'bayer',
        outline: true,
        outline_threshold: 128,
      });

      expect(result).toBe('job-pa-002');
    });
  });

  describe('generateTile', () => {
    it('calls generate_tile with correct params', async () => {
      backend.on('generate_tile', ({ request }: { request: { project_id: string; prompt: string; width?: number; height?: number; biome?: string; seamless?: boolean } }) => {
        expect(request.project_id).toBe('proj-456');
        expect(request.prompt).toBe('grass tile');
        expect(request.width).toBeUndefined();
        expect(request.height).toBeUndefined();
        expect(request.biome).toBeUndefined();
        expect(request.seamless).toBeUndefined();
        return 'job-tile-001';
      });

      const result = await assets.generateTile({
        project_id: 'proj-456',
        prompt: 'grass tile',
      });

      expect(result).toBe('job-tile-001');
    });

    it('includes optional params', async () => {
      backend.on('generate_tile', ({ request }: { request: { project_id: string; prompt: string; width?: number; height?: number; biome?: string; seamless?: boolean } }) => {
        expect(request.width).toBe(128);
        expect(request.height).toBe(128);
        expect(request.biome).toBe('forest');
        expect(request.seamless).toBe(true);
        return 'job-tile-002';
      });

      const result = await assets.generateTile({
        project_id: 'proj-456',
        prompt: 'forest tile',
        width: 128,
        height: 128,
        biome: 'forest',
        seamless: true,
      });

      expect(result).toBe('job-tile-002');
    });
  });
});
