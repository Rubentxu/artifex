import type { StorageBackend } from './storage-backend';
import { TauriBackend } from './tauri-backend';
import type { AssetResponse, ImportAssetRequest, GenerateImageRequest, GenerateAudioRequest, GenerateTtsRequest, RemoveBackgroundRequest, ConvertPixelArtRequest, GenerateTileRequest, GenerateSpriteSheetRequest } from '$lib/types/asset';

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
