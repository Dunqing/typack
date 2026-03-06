<script setup lang="ts">
import type { BundleOptions } from "../composables/useTypack";

const options = defineModel<BundleOptions>({ required: true });
</script>

<template>
  <div class="options-panel">
    <label class="option">
      <input v-model="options.sourcemap" type="checkbox" />
      <span>sourcemap</span>
    </label>
    <label class="option">
      <input v-model="options.cjsDefault" type="checkbox" />
      <span>cjs_default</span>
    </label>
    <div class="option-field">
      <label>external</label>
      <input
        :value="options.external.join(', ')"
        placeholder="e.g. react, vue"
        @input="
          options.external = ($event.target as HTMLInputElement).value
            .split(',')
            .map((s) => s.trim())
            .filter(Boolean)
        "
      />
    </div>
  </div>
</template>

<style scoped>
.options-panel {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 6px 16px;
  background: #1e293b;
  border-top: 1px solid #334155;
  color: #cbd5e1;
  font-size: 12px;
  flex-shrink: 0;
}
.option {
  display: flex;
  align-items: center;
  gap: 4px;
  cursor: pointer;
}
.option input[type="checkbox"] {
  cursor: pointer;
}
.option-field {
  display: flex;
  align-items: center;
  gap: 6px;
}
.option-field input {
  background: #0f172a;
  border: 1px solid #334155;
  color: #e2e8f0;
  padding: 2px 8px;
  border-radius: 3px;
  font-size: 12px;
  width: 180px;
}
.option-field input::placeholder {
  color: #64748b;
}
</style>
