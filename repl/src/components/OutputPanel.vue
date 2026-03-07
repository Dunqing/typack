<script setup lang="ts">
import loader from "@monaco-editor/loader";
import { ref, watch, onMounted, onBeforeUnmount } from "vue";

import { useTheme } from "../composables/useTheme";

const { monacoTheme } = useTheme();

const props = defineProps<{
  code: string;
  map: string | null;
  diagnostics: Array<{ message: string; severity: string }>;
}>();

const activeTab = ref<"output" | "diagnostics">("output");
const editorContainer = ref<HTMLDivElement>();
let editor: any = null;
let monaco: any = null;

onMounted(async () => {
  monaco = await loader.init();
  if (!editorContainer.value) return;
  editor = monaco.editor.create(editorContainer.value, {
    value: props.code,
    language: "typescript",
    theme: monacoTheme.value,
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

// Watch for theme changes and update Monaco
watch(monacoTheme, (theme) => {
  monaco?.editor.setTheme(theme);
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
  const chunks: string[] = [];
  for (let i = 0; i < bytes.length; i += 0x8000) {
    const slice = bytes.subarray(i, i + 0x8000);
    chunks.push(String.fromCharCode.apply(null, slice as unknown as number[]));
  }
  return btoa(chunks.join(""));
}

function openSourceMapViz() {
  if (!props.map || !props.code) return;
  const url = `https://evanw.github.io/source-map-visualization/#${utf8ToBase64(
    `${props.code.length}\0${props.code}${props.map.length}\0${props.map}`,
  )}`;
  window.open(url, "_blank", "noopener,noreferrer");
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div
      class="flex shrink-0 border-b border-slate-300 bg-slate-100 dark:border-neutral-700 dark:bg-neutral-900"
    >
      <button
        class="flex cursor-pointer items-center gap-1.5 border-none bg-transparent px-3 py-1.5 text-xs transition-colors"
        :class="
          activeTab === 'output'
            ? 'border-b-2 border-b-blue-500 text-slate-900 dark:text-white'
            : 'text-slate-500 hover:text-slate-700 dark:text-neutral-500 dark:hover:text-neutral-300'
        "
        @click="activeTab = 'output'"
      >
        Output
      </button>
      <button
        class="flex cursor-pointer items-center gap-1.5 border-none bg-transparent px-3 py-1.5 text-xs transition-colors"
        :class="
          activeTab === 'diagnostics'
            ? 'border-b-2 border-b-blue-500 text-slate-900 dark:text-white'
            : 'text-slate-500 hover:text-slate-700 dark:text-neutral-500 dark:hover:text-neutral-300'
        "
        @click="activeTab = 'diagnostics'"
      >
        Diagnostics
        <span
          v-if="diagnostics.length"
          class="rounded-full bg-red-500 px-1.5 py-px text-[10px] text-white"
        >
          {{ diagnostics.length }}
        </span>
      </button>
      <button
        v-if="map"
        class="flex cursor-pointer items-center gap-1.5 border-none bg-transparent px-3 py-1.5 text-xs text-slate-500 transition-colors hover:text-slate-700 dark:text-neutral-500 dark:hover:text-neutral-300"
        @click="openSourceMapViz"
      >
        Visualize Source Map
      </button>
    </div>
    <div v-show="activeTab === 'output'" ref="editorContainer" class="flex-1" />
    <div
      v-show="activeTab === 'diagnostics'"
      class="flex-1 overflow-y-auto bg-white p-2 text-slate-800 dark:bg-neutral-900 dark:text-neutral-300"
    >
      <div
        v-if="!diagnostics.length"
        class="py-6 text-center text-sm text-slate-400 dark:text-neutral-600"
      >
        No diagnostics
      </div>
      <div
        v-for="(d, i) in diagnostics"
        :key="i"
        class="flex items-start gap-2 border-b border-slate-200 p-2 text-sm dark:border-neutral-700"
      >
        <span
          class="shrink-0 rounded px-1.5 py-px text-[10px] uppercase"
          :class="d.severity === 'error' ? 'bg-red-500 text-white' : 'bg-amber-400 text-black'"
        >
          {{ d.severity }}
        </span>
        <span class="break-words">{{ d.message }}</span>
      </div>
    </div>
  </div>
</template>
