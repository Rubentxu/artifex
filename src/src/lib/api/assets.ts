import type { StorageBackend } from './storage-backend';
import { TauriBackend } from './tauri-backend';
import type { AssetResponse, ImportAssetRequest, GenerateImageRequest, GenerateAudioRequest, GenerateTtsRequest, RemoveBackgroundRequest, ConvertPixelArtRequest, GenerateTileRequest, GenerateSpriteSheetRequest, SliceSpriteSheetRequest, GenerateCodeRequest, InpaintRequest, OutpaintRequest, GenerateMaterialRequest, AnimationResponse, CreateAnimationRequest, UpdateAnimationRequest, ExportAnimationRequest, PackAtlasRequest, SeamlessTextureRequest, GenerateVideoRequest, QuickSpritesRequest, ExportProjectRequest, ExportProjectResponse, Render3dRequest } from '$lib/types/asset';

let backend: StorageBackend;
export function setBackend(b: StorageBackend): void {
  backend = b;
}

function getBackend(): StorageBackend {
  if (!backend) backend = new TauriBackend();
  return backend;
}

export async function listAssets(projectId: string): Promise<AssetResponse[]> {
  return getBackend().invoke<AssetResponse[]>('list_assets', { projectId });
}

export async function getAsset(id: string): Promise<AssetResponse> {
  return getBackend().invoke<AssetResponse>('get_asset', { id });
}

export async function deleteAsset(id: string): Promise<void> {
  return getBackend().invoke<void>('delete_asset', { id });
}

export async function importAsset(request: ImportAssetRequest): Promise<AssetResponse> {
  return getBackend().invoke<AssetResponse>('import_asset', { request });
}

export async function generateImage(request: GenerateImageRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_image', { request });
}

export async function generateAudio(request: GenerateAudioRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_audio', { request });
}

export async function synthesizeSpeech(request: GenerateTtsRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('synthesize_speech', { request });
}

export async function removeBackground(request: RemoveBackgroundRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('remove_background', { request });
}

export async function convertPixelArt(request: ConvertPixelArtRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('convert_pixel_art', { request });
}

export async function generateTile(request: GenerateTileRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_tile', { request });
}

export async function generateSpriteSheet(request: GenerateSpriteSheetRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_sprite_sheet', { request });
}

export async function sliceSpriteSheet(request: SliceSpriteSheetRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('slice_sprite_sheet', { request });
}

export async function generateCode(request: GenerateCodeRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_code', { request });
}

export async function listCodeTemplates(engine: string): Promise<import('$lib/types/asset').CodeTemplate[]> {
  return getBackend().invoke<import('$lib/types/asset').CodeTemplate[]>('list_code_templates', { engine });
}

export async function inpaintImage(request: InpaintRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('inpaint_image', { request });
}

export async function outpaintImage(request: OutpaintRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('outpaint_image', { request });
}

export async function generateMaterial(request: GenerateMaterialRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_material', { request });
}

// Animation API functions

export async function createAnimation(request: CreateAnimationRequest): Promise<string> {
  // Returns asset_id
  return getBackend().invoke<string>('create_animation', { request });
}

export async function getAnimation(id: string): Promise<AnimationResponse> {
  return getBackend().invoke<AnimationResponse>('get_animation', { id });
}

export async function listAnimations(projectId: string): Promise<AnimationResponse[]> {
  return getBackend().invoke<AnimationResponse[]>('list_animations', { projectId });
}

export async function updateAnimation(request: UpdateAnimationRequest): Promise<string> {
  // Returns asset_id
  return getBackend().invoke<string>('update_animation', { request });
}

export async function deleteAnimation(id: string): Promise<void> {
  return getBackend().invoke<void>('delete_animation', { id });
}

export async function exportAnimation(request: ExportAnimationRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('export_animation', { request });
}

export async function packAtlas(request: PackAtlasRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('pack_atlas', { request });
}

export async function generateSeamlessTexture(request: SeamlessTextureRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_seamless_texture', { request });
}

export async function generateVideo(request: GenerateVideoRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_video', { request });
}

export async function generateQuickSprites(request: QuickSpritesRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('generate_quick_sprites', { request });
}

export async function exportProject(request: ExportProjectRequest): Promise<ExportProjectResponse> {
  return getBackend().invoke<ExportProjectResponse>('export_project', { request });
}

export async function openItchIo(): Promise<void> {
  return getBackend().invoke<void>('open_itch_io');
}

export async function render3dToSprites(request: Render3dRequest): Promise<string> {
  // Returns job_id
  return getBackend().invoke<string>('render_3d_to_sprites', { request });
}
