<script setup lang="ts">
import loader from "@monaco-editor/loader";
import { ref, watch, onMounted, onBeforeUnmount } from "vue";

const props = defineProps<{
  code: string;
  map: string | null;
  diagnostics: Array<{ message: string; severity: string }>;
}>();

const activeTab = ref<"output" | "diagnostics" | "sourcemap">("output");
const editorContainer = ref<HTMLDivElement>();
let editor: any = null;

onMounted(async () => {
  const monaco = await loader.init();
  if (!editorContainer.value) return;
  editor = monaco.editor.create(editorContainer.value, {
    value: props.code,
    language: "typescript",
    theme: "vs-dark",
    minimap: { enabled: false },
    fontSize: 13,
    readOnly: true,
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 2,
  });
});

onBeforeUnmount(() => {
  editor?.dispose();
});

watch(
  () => props.code,
  (val) => {
    if (editor && editor.getValue() !== val) {
      editor.setValue(val);
    }
  },
);

function utf8ToBase64(input: string): string {
  const bytes = new TextEncoder().encode(input);
  let binary = "";
  for (let i = 0; i < bytes.length; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

function openSourceMapViz() {
  if (!props.map || !props.code) return;
  const url = `https://evanw.github.io/source-map-visualization/#${utf8ToBase64(
    `${props.code.length}\0${props.code}${props.map.length}\0${props.map}`,
  )}`;
  window.open(url, "_blank");
}
</script>

<template>
  <div class="output-panel">
    <div class="output-tabs">
      <button
        class="output-tab"
        :class="{ active: activeTab === 'output' }"
        @click="activeTab = 'output'"
      >
        Output
      </button>
      <button
        class="output-tab"
        :class="{ active: activeTab === 'diagnostics' }"
        @click="activeTab = 'diagnostics'"
      >
        Diagnostics
        <span v-if="diagnostics.length" class="badge">{{ diagnostics.length }}</span>
      </button>
      <button v-if="map" class="output-tab" @click="openSourceMapViz">Visualize Source Map</button>
    </div>
    <div v-show="activeTab === 'output'" ref="editorContainer" class="editor-container" />
    <div v-show="activeTab === 'diagnostics'" class="diagnostics">
      <div v-if="!diagnostics.length" class="empty">No diagnostics</div>
      <div v-for="(d, i) in diagnostics" :key="i" class="diagnostic" :class="d.severity">
        <span class="severity-badge">{{ d.severity }}</span>
        <span class="message">{{ d.message }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.output-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.output-tabs {
  display: flex;
  background: #1e1e1e;
  border-bottom: 1px solid #333;
  flex-shrink: 0;
}
.output-tab {
  padding: 6px 12px;
  color: #999;
  cursor: pointer;
  font-size: 12px;
  border: none;
  background: none;
  display: flex;
  align-items: center;
  gap: 6px;
}
.output-tab.active {
  color: #fff;
  border-bottom: 2px solid #3b82f6;
}
.output-tab:hover {
  color: #ddd;
}
.badge {
  background: #ef4444;
  color: white;
  font-size: 10px;
  padding: 1px 5px;
  border-radius: 8px;
}
.editor-container {
  flex: 1;
}
.diagnostics {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  background: #1e1e1e;
  color: #ddd;
}
.empty {
  color: #666;
  text-align: center;
  padding: 24px;
  font-size: 13px;
}
.diagnostic {
  padding: 8px;
  border-bottom: 1px solid #333;
  font-size: 13px;
  display: flex;
  align-items: flex-start;
  gap: 8px;
}
.severity-badge {
  font-size: 10px;
  padding: 1px 6px;
  border-radius: 4px;
  text-transform: uppercase;
  flex-shrink: 0;
}
.diagnostic.error .severity-badge {
  background: #ef4444;
}
.diagnostic.warning .severity-badge {
  background: #f59e0b;
  color: #000;
}
.message {
  word-break: break-word;
}
</style>
