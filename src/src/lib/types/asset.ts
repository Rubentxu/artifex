// Asset types matching Rust DTOs (camelCase for JSON)

export type AssetKind = 'Image' | 'Sprite' | 'Tileset' | 'Material' | 'Audio' | 'Voice' | 'Video' | 'Other';

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
