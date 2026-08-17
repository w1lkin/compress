<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import { checkGs, listPresets, compressFiles } from "./api";
import type { FileResult, Preset } from "./types";
import PresetPicker from "./components/PresetPicker.vue";
import OutputDirPicker from "./components/OutputDirPicker.vue";
import FileList from "./components/FileList.vue";

const STORAGE_KEY = "pdf-compressor.outputDir";

const presets = ref<Preset[]>([]);
const presetId = ref("");
const outputDir = ref("");
const files = ref<string[]>([]);
const results = ref<FileResult[]>([]);
const rasterize = ref(false);
const gsMsg = ref("");
const busy = ref(false);

onMounted(async () => {
  // 恢复上次的输出目录
  const saved = localStorage.getItem(STORAGE_KEY);
  if (saved) outputDir.value = saved;

  const gs = await checkGs();
  if (gs.found) {
    gsMsg.value = `Ghostscript ${gs.version}`;
  } else {
    gsMsg.value = gs.install_hint;
  }
  presets.value = await listPresets("pdf");
  if (presets.value.length) presetId.value = presets.value[0].id;
});

// 输出目录变化时持久化
watch(outputDir, (v) => {
  if (v) localStorage.setItem(STORAGE_KEY, v);
  else localStorage.removeItem(STORAGE_KEY);
});

async function addFiles() {
  const selected = await open({ multiple: true, filters: [{ name: "PDF", extensions: ["pdf"] }] });
  if (Array.isArray(selected)) files.value.push(...(selected as string[]));
  else if (typeof selected === "string") files.value.push(selected);
}

function removeFile(i: number) {
  files.value.splice(i, 1);
}

function fileName(p: string) {
  return p.split(/[\\/]/).pop() ?? p;
}

async function start() {
  if (!files.value.length || !presetId.value || !outputDir.value) return;
  busy.value = true;
  results.value = [];
  results.value = await compressFiles(files.value, presetId.value, {
    rasterize_text_layer: rasterize.value,
    output_dir: outputDir.value,
  });
  files.value = [];
  busy.value = false;
}
</script>

<template>
  <div class="container">
    <header>
      <h1>PDF 压缩器</h1>
      <span class="gs">{{ gsMsg }}</span>
    </header>

    <section class="controls">
      <button type="button" @click="addFiles">添加 PDF 文件…</button>
      <PresetPicker v-model="presetId" :presets="presets" />
      <OutputDirPicker v-model="outputDir" />
      <label class="raster">
        <input type="checkbox" v-model="rasterize" :disabled="presetId !== 'extreme'" />
        栅格化去除文字层（仅极致压缩可用）
      </label>
      <button type="button" class="primary" :disabled="busy || !files.length" @click="start">
        {{ busy ? "压缩中…" : "开始压缩" }}
      </button>
    </section>

    <section v-if="files.length" class="queue">
      <h2>待压缩文件（{{ files.length }}）</h2>
      <ul>
        <li v-for="(f, i) in files" :key="f">
          <span class="fname">{{ fileName(f) }}</span>
          <span class="fpath">{{ f }}</span>
          <button type="button" class="remove" @click="removeFile(i)">移除</button>
        </li>
      </ul>
    </section>

    <FileList :items="results" />
  </div>
</template>

<style>
body { font-family: system-ui, -apple-system, sans-serif; margin: 0; background: #f5f6f8; }
.container { padding: 24px; max-width: 1080px; margin: 0 auto; }
header { display: flex; align-items: baseline; gap: 16px; }
h1 { margin: 0 0 16px; }
.gs { color: #666; font-size: 13px; }
.controls { display: flex; flex-direction: column; gap: 16px; margin-bottom: 24px; }
.raster { font-size: 13px; color: #444; }
button { padding: 8px 16px; cursor: pointer; }
.primary { background: #2563eb; color: #fff; border: none; border-radius: 6px; }
.primary:disabled { background: #9ca3af; cursor: not-allowed; }
.queue { margin-bottom: 24px; }
.queue h2 { font-size: 15px; margin: 0 0 10px; }
.queue ul { list-style: none; padding: 0; margin: 0; }
.queue li { display: flex; align-items: center; gap: 12px; padding: 6px 10px; background: #fff; border: 1px solid #e5e7eb; border-radius: 6px; margin-bottom: 6px; }
.queue .fname { font-weight: 600; }
.queue .fpath { flex: 1; color: #888; font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.queue .remove { padding: 2px 8px; font-size: 12px; color: #dc2626; }
</style>
