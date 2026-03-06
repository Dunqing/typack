<script setup lang="ts">
defineProps<{
  loading: boolean;
  ready: boolean;
  bundleTime: number;
}>();
</script>

<template>
  <header class="header">
    <div class="header-left">
      <h1 class="title">Typack REPL</h1>
      <span v-if="!ready" class="status loading">Loading WASM...</span>
      <span v-else-if="loading" class="status bundling">Bundling...</span>
      <span v-else class="status ready"
        >Ready<template v-if="bundleTime > 0"> · {{ bundleTime }}ms</template></span
      >
    </div>
    <div class="header-right">
      <a
        href="https://github.com/Dunqing/typack"
        target="_blank"
        rel="noopener noreferrer"
        class="github-link"
        title="View on GitHub"
      >
        <svg viewBox="0 0 16 16" width="20" height="20" fill="currentColor">
          <path
            d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38
            0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52
            -.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2
            -3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82
            .64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08
            2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01
            1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"
          />
        </svg>
      </a>
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

.github-link {
  color: #f1f5f9;
  opacity: 0.7;
  transition: opacity 0.15s;
  display: flex;
  align-items: center;
}

.github-link:hover {
  opacity: 1;
}
</style>
