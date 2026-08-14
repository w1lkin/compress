<script setup lang="ts">
import type { Preset } from "../types";

defineProps<{ presets: Preset[]; modelValue: string }>();
const emit = defineEmits<{ (e: "update:modelValue", v: string): void }>();
</script>

<template>
  <div class="preset-picker">
    <label v-for="p in presets" :key="p.id" class="preset">
      <input
        type="radio"
        :value="p.id"
        :checked="p.id === modelValue"
        @change="emit('update:modelValue', p.id)"
      />
      {{ p.label }}
    </label>
  </div>
</template>

<style scoped>
.preset-picker { display: flex; gap: 16px; }
.preset { cursor: pointer; }
</style>
