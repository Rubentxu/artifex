import { writable, derived } from 'svelte/store';
import type { AssetResponse, AssetKind, GenerateImageRequest, GenerateAudioRequest, GenerateTtsRequest, RemoveBackgroundRequest, ConvertPixelArtRequest, GenerateTileRequest } from '$lib/types/asset';
import * as assetApi from '$lib/api/assets';

interface AssetState {
  assets: AssetResponse[];
  selectedId: string | null;
  loading: boolean;
  error: string | null;
  filterKind: AssetKind | null;
}

function createAssetStore() {
  const { subscribe, set, update } = writable<AssetState>({
    assets: [],
    selectedId: null,
    loading: false,
    error: null,
    filterKind: null,
  });

  return {
    subscribe,

    setFilterKind(kind: AssetKind | null) {
      update(s => ({ ...s, filterKind: kind }));
    },

    async loadAssets(projectId: string) {
      update(s => ({ ...s, loading: true, error: null }));
      try {
        const assets = await assetApi.listAssets(projectId);
        update(s => ({ ...s, assets, loading: false }));
      } catch (e) {
        update(s => ({ ...s, error: String(e), loading: false }));
      }
    },

    selectAsset(id: string | null) {
      update(s => ({ ...s, selectedId: id }));
    },

    async deleteAsset(id: string) {
      await assetApi.deleteAsset(id);
      update(s => ({
        ...s,
        assets: s.assets.filter(a => a.id !== id),
        selectedId: s.selectedId === id ? null : s.selectedId,
      }));
    },

    async importAsset(projectId: string, sourcePath: string, name: string, kind: string) {
      const asset = await assetApi.importAsset({ project_id: projectId, source_path: sourcePath, name, kind });
      update(s => ({ ...s, assets: [asset, ...s.assets] }));
      return asset;
    },

    async generateImage(request: GenerateImageRequest) {
      const jobId = await assetApi.generateImage(request);
      return jobId;
    },

    async generateAudio(request: GenerateAudioRequest) {
      const jobId = await assetApi.generateAudio(request);
      return jobId;
    },

    async synthesizeSpeech(request: GenerateTtsRequest) {
      const jobId = await assetApi.synthesizeSpeech(request);
      return jobId;
    },

    async removeBackground(request: RemoveBackgroundRequest) {
      const jobId = await assetApi.removeBackground(request);
      return jobId;
    },

    async convertPixelArt(request: ConvertPixelArtRequest) {
      const jobId = await assetApi.convertPixelArt(request);
      return jobId;
    },

    async generateTile(request: GenerateTileRequest) {
      const jobId = await assetApi.generateTile(request);
      return jobId;
    },
  };
}

export const assetStore = createAssetStore();

export const selectedAsset = derived(assetStore, ($state) =>
  $state.assets.find(a => a.id === $state.selectedId) ?? null
);

export const filteredAssets = derived(assetStore, ($state) =>
  $state.filterKind ? $state.assets.filter(a => a.kind === $state.filterKind) : $state.assets
);
