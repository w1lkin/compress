export interface GhostscriptInfo {
  found: boolean;
  path: string | null;
  version: string | null;
  install_hint: string;
}

export interface Preset {
  id: string;
  label: string;
  gs_settings: string;
  rasterize: boolean;
}

export interface CompressOptions {
  rasterize_text_layer: boolean;
  output_dir: string;
}

export interface CompressionResult {
  input_path: string;
  output_path: string;
  original_size: number;
  compressed_size: number;
  page_count: number;
  preset: string;
  text_layer_preserved: boolean;
  duration_ms: number;
}

export interface FileResult {
  success: boolean;
  input_path?: string;
  output_path?: string;
  original_size?: number;
  compressed_size?: number;
  page_count?: number;
  preset?: string;
  text_layer_preserved?: boolean;
  duration_ms?: number;
  error?: string | null;
}
