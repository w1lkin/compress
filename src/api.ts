import { invoke } from "@tauri-apps/api/core";
import type {
  CompressOptions,
  FileResult,
  GhostscriptInfo,
  Preset,
} from "./types";

export const checkGs = () => invoke<GhostscriptInfo>("check_gs");
export const listPresets = (ext: string) => invoke<Preset[]>("list_presets", { ext });
export const compressFiles = (
  paths: string[],
  presetId: string,
  opts: CompressOptions,
) =>
  invoke<FileResult[]>("compress_files", {
    paths,
    presetId,
    opts,
  });
