// Asset types matching Rust DTOs (camelCase for JSON)

export type AssetKind = 'Image' | 'Sprite' | 'Tileset' | 'Material' | 'Audio' | 'Voice' | 'Video' | 'Code' | 'Animation' | 'Other';

export type CodeEngine = 'godot' | 'unity';

export interface GenerateCodeRequest {
  projectId: string;
  engine: CodeEngine;
  prompt: string;
  templateId?: string;
  modelId?: string;
  temperature?: number;
  maxTokens?: number;
}

export interface CodeFileOutput {
  path: string;
  language: string;
  description: string;
  content: string;
}

export interface CodeTemplate {
  id: string;
  name: string;
  description: string;
  engine: string;
  variables: string[];
}

export interface AssetResponse {
  id: string;
  project_id: string;
  name: string;
  kind: AssetKind;
  file_path: string | null;
  metadata: Record<string, unknown> | null;
  file_size: number | null;
  width: number | null;
  height: number | null;
  duration_secs?: number | null;
  sample_rate?: number | null;
  created_at: string;
}

export interface ImportAssetRequest {
  project_id: string;
  source_path: string;
  name: string;
  kind: string;
}

export interface GenerateImageRequest {
  project_id: string;
  prompt: string;
  negative_prompt?: string;
  width: number;
  height: number;
  steps: number;
  seed?: number;
  model_id?: string;
}

export interface GenerateAudioRequest {
  project_id: string;
  params: AudioGenParams;
}

export interface AudioGenParams {
  prompt: string;
  kind?: 'sfx' | 'music';
  duration_secs?: number;
  model_id?: string;
  seed?: number;
  output_format?: string;
}

export interface GenerateTtsRequest {
  project_id: string;
  params: TtsParams;
}

export interface TtsParams {
  text: string;
  voice_id?: string;
  language?: string;
  speed?: number;
  model_id?: string;
  stability?: number;
  similarity_boost?: number;
  output_format?: string;
}

export interface RemoveBackgroundRequest {
  project_id: string;
  asset_id: string;
  provider_mode?: string;
}

export interface ConvertPixelArtRequest {
  project_id: string;
  asset_id: string;
  target_width: number;
  target_height: number;
  palette: PaletteMode;
  dithering: DitheringMode;
  outline: boolean;
  outline_threshold?: number;
}

export type PaletteMode =
  | { type: 'Pico8' }
  | { type: 'GameBoy' }
  | { type: 'Nes' }
  | { type: 'Custom'; colors: [number, number, number][] };

export type DitheringMode = 'none' | 'floyd_steinberg' | 'bayer' | 'atkinson';

export interface GenerateTileRequest {
  project_id: string;
  prompt: string;
  width?: number;
  height?: number;
  biome?: string;
  seamless?: boolean;
}

export type OutputFormat = 'Json' | 'Aseprite' | 'Both';

export interface GenerateSpriteSheetRequest {
  asset_id: string;
  project_id: string;
  fps?: number;
  dedup_threshold?: number;
  atlas_max_size?: number;
  padding?: number;
  animation_name?: string;
  output_format?: OutputFormat;
}

export type SliceMode = 'Grid' | 'AutoDetect';

export type SortOrder = 'LeftToRight' | 'TopToBottom';

export interface GridSliceParams {
  rows?: number;
  cols?: number;
  margin?: number;
}

export interface AutoDetectSliceParams {
  min_area?: number;
  sort_order?: SortOrder;
}

export interface SliceSpriteSheetRequest {
  asset_id: string;
  project_id: string;
  mode?: SliceMode;
  grid_params?: GridSliceParams;
  auto_detect_params?: AutoDetectSliceParams;
}

export type OutpaintDirection = 'left' | 'right' | 'top' | 'bottom';

export interface InpaintRequest {
  project_id: string;
  asset_id: string;
  mask_path: string;
  prompt: string;
  negative_prompt?: string;
  strength?: number;
  guidance_scale?: number;
  steps?: number;
  provider_mode?: string;
}

export interface OutpaintRequest {
  project_id: string;
  asset_id: string;
  direction: OutpaintDirection;
  extend_pixels: number;
  prompt: string;
  negative_prompt?: string;
  strength?: number;
  guidance_scale?: number;
  steps?: number;
  provider_mode?: string;
}

export type MapKind = 'basecolor' | 'normal' | 'roughness' | 'metalness' | 'height';

export interface GenerateMaterialRequest {
  project_id: string;
  asset_id: string;
  provider_id?: string;
  model_id?: string;
}

// Animation types

export interface AnimationMetadata {
  name: string;
  frame_asset_ids: string[];
  frame_durations_ms: number[];
  loop_animation: boolean;
  total_duration_ms: number;
  default_fps?: number;
}

export interface AnimationResponse {
  id: string;
  project_id: string;
  name: string;
  kind: 'animation';
  metadata: AnimationMetadata | null;
  created_at: string;
}

export interface CreateAnimationRequest {
  project_id: string;
  name: string;
  frame_asset_ids: string[];
  default_fps?: number;
}

export interface UpdateAnimationRequest {
  id: string;
  name?: string;
  frame_asset_ids?: string[];
  frame_durations_ms?: number[];
  loop_animation?: boolean;
}

export interface ExportAnimationRequest {
  animation_id: string;
  project_id: string;
  format?: 'spritesheet_json';
}

// Atlas packing types

export type AtlasSortMode = 'Area' | 'MaxSide' | 'Width' | 'Height' | 'None';

export interface PackAtlasOptions {
  max_size: 512 | 1024 | 2048 | 4096;
  padding: number;       // 0-16
  allow_rotation: boolean;
  sort_mode: AtlasSortMode;
}

export interface PackAtlasRequest {
  project_id: string;
  atlas_name: string;
  source_asset_ids: string[];
  options: PackAtlasOptions;
}

export interface AtlasRegion {
  asset_id: string;
  name: string;
  x: number;
  y: number;
  w: number;
  h: number;
  source_width: number;
  source_height: number;
  rotated: boolean;
}

export interface AtlasManifest {
  version: number;
  atlas_name: string;
  atlas_width: number;
  atlas_height: number;
  regions: AtlasRegion[];
}

// Seamless Texture Generation types

export type SeamlessMode = 'FromPrompt' | 'FromAsset';

export interface SeamlessTextureRequest {
  project_id: string;
  mode: SeamlessMode;
  prompt?: string;
  negative_prompt?: string;
  width?: number;
  height?: number;
  asset_id?: string;
  secondary_asset_id?: string;
  seam_threshold?: number;
  padding_pixels?: number;
  blend_fraction?: number;
}
