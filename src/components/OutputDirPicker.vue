<script setup lang="ts">
import { open } from "@tauri-apps/plugin-dialog";

defineProps<{ modelValue: string }>();
const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();

async function pick() {
  const dir = await open({ directory: true });
  if (typeof dir === "string") emit("update:modelValue", dir);
}
</script>

<template>
  <div class="output-picker">
    <span>输出目录：</span>
    <input :value="modelValue" readonly class="path" />
    <button type="button" @click="pick">选择…</button>
  </div>
</template>

<style scoped>
.output-picker { display: flex; gap: 8px; align-items: center; }
.path { flex: 1; }
</style>
