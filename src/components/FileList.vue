<script setup lang="ts">
import type { FileResult } from "../types";

defineProps<{ items: FileResult[] }>();

function fmt(bytes?: number) {
  if (bytes == null) return "-";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / 1024 / 1024).toFixed(1)} MB`;
}

function ratio(r: FileResult) {
  if (!r.success || r.original_size == null || r.compressed_size == null) return "-";
  return `${(((r.compressed_size - r.original_size) / r.original_size) * 100).toFixed(1)}%`;
}
</script>

<template>
  <table class="file-list">
    <thead>
      <tr>
        <th>文件</th><th>原大小</th><th>新大小</th><th>压缩率</th>
        <th>页数</th><th>文字层</th><th>耗时</th><th>状态</th>
      </tr>
    </thead>
    <tbody>
      <tr v-for="(r, i) in items" :key="i">
        <td>{{ r.input_path }}</td>
        <td>{{ fmt(r.original_size) }}</td>
        <td>{{ fmt(r.compressed_size) }}</td>
        <td>{{ ratio(r) }}</td>
        <td>{{ r.page_count ?? "-" }}</td>
        <td>{{ r.text_layer_preserved == null ? "-" : r.text_layer_preserved ? "可搜索" : "已栅格化" }}</td>
        <td>{{ r.duration_ms != null ? r.duration_ms + " ms" : "-" }}</td>
        <td :class="r.success ? 'ok' : 'fail'">{{ r.success ? "成功" : r.error }}</td>
      </tr>
    </tbody>
  </table>
</template>

<style scoped>
.file-list { width: 100%; border-collapse: collapse; }
th, td { border: 1px solid #ddd; padding: 6px 10px; text-align: left; font-size: 13px; }
.ok { color: #16a34a; }
.fail { color: #dc2626; }
</style>
