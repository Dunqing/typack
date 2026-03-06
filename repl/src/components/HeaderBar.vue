<script setup lang="ts">
defineProps<{
  loading: boolean;
  ready: boolean;
}>();

defineEmits<{
  bundle: [];
}>();
</script>

<template>
  <header class="header">
    <div class="header-left">
      <h1 class="title">typack REPL</h1>
      <span v-if="!ready" class="status loading">Loading WASM...</span>
      <span v-else-if="loading" class="status bundling">Bundling...</span>
      <span v-else class="status ready">Ready</span>
    </div>
    <div class="header-right">
      <button class="btn" :disabled="!ready || loading" @click="$emit('bundle')">Bundle</button>
    </div>
  </header>
</template>

<style scoped>
.header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: #1e293b;
  color: #f1f5f9;
  height: 48px;
  flex-shrink: 0;
}
.header-left {
  display: flex;
  align-items: center;
  gap: 12px;
}
.title {
  font-size: 16px;
  font-weight: 600;
}
.status {
  font-size: 12px;
  padding: 2px 8px;
  border-radius: 4px;
}
.status.loading {
  background: #f59e0b;
  color: #000;
}
.status.bundling {
  background: #3b82f6;
}
.status.ready {
  background: #22c55e;
  color: #000;
}
.btn {
  padding: 6px 16px;
  border: none;
  border-radius: 4px;
  background: #3b82f6;
  color: white;
  cursor: pointer;
  font-size: 13px;
  font-weight: 500;
}
.btn:hover:not(:disabled) {
  background: #2563eb;
}
.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
