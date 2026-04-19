import { writable, derived } from 'svelte/store';
import type { AssetResponse, AssetKind, GenerateImageRequest, GenerateAudioRequest, GenerateTtsRequest, RemoveBackgroundRequest, ConvertPixelArtRequest, GenerateTileRequest, GenerateSpriteSheetRequest, SliceSpriteSheetRequest, GenerateCodeRequest, InpaintRequest, OutpaintRequest, GenerateMaterialRequest, CreateAnimationRequest, UpdateAnimationRequest, AnimationResponse, PackAtlasRequest, SeamlessTextureRequest, GenerateVideoRequest, QuickSpritesRequest, ExportProjectRequest, ExportProjectResponse, Render3dRequest, CollectionResponse } from '$lib/types/asset';
import * as assetApi from '$lib/api/assets';

interface AssetState {
  assets: AssetResponse[];
  selectedId: string | null;
  loading: boolean;
  error: string | null;
  filterKind: AssetKind | null;
  filterTags: string[];
  filterCollection: string | null;
  collections: CollectionResponse[];
}

function createAssetStore() {
  const { subscribe, set, update } = writable<AssetState>({
    assets: [],
    selectedId: null,
    loading: false,
    error: null,
    filterKind: null,
    filterTags: [],
    filterCollection: null,
    collections: [],
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

    async generateSpriteSheet(request: GenerateSpriteSheetRequest) {
      const jobId = await assetApi.generateSpriteSheet(request);
      return jobId;
    },

    async sliceSpriteSheet(request: SliceSpriteSheetRequest) {
      const jobId = await assetApi.sliceSpriteSheet(request);
      return jobId;
    },

    async generateCode(request: GenerateCodeRequest) {
      const jobId = await assetApi.generateCode(request);
      return jobId;
    },

    async inpaintImage(request: InpaintRequest) {
      const jobId = await assetApi.inpaintImage(request);
      return jobId;
    },

    async outpaintImage(request: OutpaintRequest) {
      const jobId = await assetApi.outpaintImage(request);
      return jobId;
    },

    async generateMaterial(request: GenerateMaterialRequest) {
      const jobId = await assetApi.generateMaterial(request);
      return jobId;
    },

    async createAnimation(request: CreateAnimationRequest) {
      const assetId = await assetApi.createAnimation(request);
      // Refresh asset list after creating animation
      const projectId = request.project_id;
      const assets = await assetApi.listAssets(projectId);
      update(s => ({ ...s, assets }));
      return assetId;
    },

    async updateAnimation(request: UpdateAnimationRequest) {
      await assetApi.updateAnimation(request);
      // Refresh asset list after updating animation
      const assets = await assetApi.listAssets(request.id.split('-')[0]); // rough project id extraction
      update(s => ({ ...s, assets }));
      return request.id;
    },

    async deleteAnimation(id: string) {
      await assetApi.deleteAnimation(id);
      update(s => ({
        ...s,
        assets: s.assets.filter(a => a.id !== id),
        selectedId: s.selectedId === id ? null : s.selectedId,
      }));
    },

    async exportAnimation(request: { animation_id: string; project_id: string }) {
      const jobId = await assetApi.exportAnimation(request);
      return jobId;
    },

    async packAtlas(request: PackAtlasRequest) {
      const jobId = await assetApi.packAtlas(request);
      return jobId;
    },

    async generateSeamlessTexture(request: SeamlessTextureRequest) {
      const jobId = await assetApi.generateSeamlessTexture(request);
      return jobId;
    },

    async generateVideo(request: GenerateVideoRequest) {
      const jobId = await assetApi.generateVideo(request);
      return jobId;
    },

    async generateQuickSprites(request: QuickSpritesRequest) {
      const jobId = await assetApi.generateQuickSprites(request);
      return jobId;
    },

    async exportProject(request: ExportProjectRequest): Promise<ExportProjectResponse> {
      return await assetApi.exportProject(request);
    },

    async openItchIo(): Promise<void> {
      await assetApi.openItchIo();
    },

    async render3d(request: Render3dRequest) {
      const jobId = await assetApi.render3dToSprites(request);
      return jobId;
    },

    setFilterTags(tags: string[]) {
      update(s => ({ ...s, filterTags: tags }));
    },

    toggleTagFilter(tag: string) {
      update(s => {
        const tags = s.filterTags.includes(tag)
          ? s.filterTags.filter(t => t !== tag)
          : [...s.filterTags, tag];
        return { ...s, filterTags: tags };
      });
    },

    setFilterCollection(collectionId: string | null) {
      update(s => ({ ...s, filterCollection: collectionId }));
    },

    async loadCollections(projectId: string) {
      try {
        const collections = await assetApi.listCollections(projectId);
        update(s => ({ ...s, collections }));
      } catch (e) {
        console.error('Failed to load collections:', e);
      }
    },

    async tagAsset(assetId: string, tag: string) {
      const asset = await assetApi.tagAsset({ asset_id: assetId, tag });
      update(s => ({
        ...s,
        assets: s.assets.map(a => a.id === assetId ? asset : a),
      }));
      return asset;
    },

    async untagAsset(assetId: string, tag: string) {
      const asset = await assetApi.untagAsset({ asset_id: assetId, tag });
      update(s => ({
        ...s,
        assets: s.assets.map(a => a.id === assetId ? asset : a),
      }));
      return asset;
    },

    async createCollection(projectId: string, name: string) {
      const collection = await assetApi.createCollection({ project_id: projectId, name });
      update(s => ({
        ...s,
        collections: [...s.collections, collection],
      }));
      return collection;
    },

    async deleteCollection(collectionId: string) {
      await assetApi.deleteCollection(collectionId);
      update(s => ({
        ...s,
        collections: s.collections.filter(c => c.id !== collectionId),
        // Clear filter if deleted collection was selected
        filterCollection: s.filterCollection === collectionId ? null : s.filterCollection,
      }));
    },

    async addToCollection(assetId: string, collectionId: string) {
      const asset = await assetApi.addToCollection({ asset_id: assetId, collection_id: collectionId });
      update(s => ({
        ...s,
        assets: s.assets.map(a => a.id === assetId ? asset : a),
      }));
      return asset;
    },

    async removeFromCollection(assetId: string) {
      const asset = await assetApi.removeFromCollection(assetId);
      update(s => ({
        ...s,
        assets: s.assets.map(a => a.id === assetId ? asset : a),
      }));
      return asset;
    },
  };
}

export const assetStore = createAssetStore();

export const selectedAsset = derived(assetStore, ($state) =>
  $state.assets.find(a => a.id === $state.selectedId) ?? null
);

export const filteredAssets = derived(assetStore, ($state) => {
  let assets = $state.assets;

  // Filter by kind
  if ($state.filterKind) {
    assets = assets.filter(a => a.kind === $state.filterKind);
  }

  // Filter by collection
  if ($state.filterCollection) {
    assets = assets.filter(a => a.collection_id === $state.filterCollection);
  }

  // Filter by tags (AND logic - asset must have all selected tags)
  if ($state.filterTags.length > 0) {
    assets = assets.filter(a =>
      $state.filterTags.every(tag => a.tags.includes(tag))
    );
  }

  return assets;
});
