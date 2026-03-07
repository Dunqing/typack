<script setup lang="ts">
import loader from "@monaco-editor/loader";
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";

import { useTheme } from "../composables/useTheme";
import type { BundleOutput } from "../types";

const { monacoTheme } = useTheme();

const props = defineProps<{
  output: BundleOutput[];
  entryNames: string[];
  diagnostics: Array<{ message: string; severity: string }>;
}>();

const activeTab = ref<"output" | "diagnostics">("output");
const activeOutputIdx = ref(0);

const currentOutput = computed(() => props.output[activeOutputIdx.value]);
const currentCode = computed(() => currentOutput.value?.code ?? "");
const currentMap = computed(() => currentOutput.value?.map ?? null);

watch(
  () => props.output.length,
  (len) => {
    if (activeOutputIdx.value >= len) {
      activeOutputIdx.value = Math.max(0, len - 1);
    }
  },
);

const editorContainer = ref<HTMLDivElement>();
let editor: any = null;
let monaco: any = null;

onMounted(async () => {
  monaco = await loader.init();
  if (!editorContainer.value) return;
  editor = monaco.editor.create(editorContainer.value, {
    value: currentCode.value,
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

watch(currentCode, (val) => {
  if (editor && editor.getValue() !== val) {
    editor.setValue(val);
  }
});

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
  if (!currentMap.value || !currentCode.value) return;
  const url = `https://evanw.github.io/source-map-visualization/#${utf8ToBase64(
    `${currentCode.value.length}\0${currentCode.value}${currentMap.value.length}\0${currentMap.value}`,
  )}`;
  window.open(url, "_blank", "noopener,noreferrer");
}
</script>

<template>
  <div class="flex h-full flex-col">
    <div
      v-if="output.length > 1"
      class="flex shrink-0 overflow-x-auto border-b border-slate-300 bg-slate-100 dark:border-neutral-700 dark:bg-neutral-900"
    >
      <button
        v-for="(_, idx) in output"
        :key="idx"
        class="cursor-pointer border-r border-none border-slate-300 bg-transparent px-3 py-1.5 text-xs whitespace-nowrap transition-colors dark:border-neutral-700"
        :class="
          idx === activeOutputIdx
            ? 'border-b-2 border-b-blue-500 text-slate-900 dark:text-white'
            : 'text-slate-500 hover:text-slate-700 dark:text-neutral-500 dark:hover:text-neutral-300'
        "
        @click="activeOutputIdx = idx"
      >
        output-{{ entryNames[idx] ?? `${idx + 1}.d.ts` }}
      </button>
    </div>
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
        v-if="currentMap"
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
