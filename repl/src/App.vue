<script setup lang="ts">
import { Splitpanes, Pane } from "splitpanes";
import { ref, onMounted, onBeforeUnmount, watch } from "vue";

import "splitpanes/dist/splitpanes.css";
import EditorPanel from "./components/EditorPanel.vue";
import HeaderBar from "./components/HeaderBar.vue";
import OutputPanel from "./components/OutputPanel.vue";
import { useFiles } from "./composables/useFiles";
import { useTheme } from "./composables/useTheme";
import { useTypack } from "./composables/useTypack";
import { useUrlState } from "./composables/useUrlState";

const { files, activeFile, addFile, removeFile, renameFile, updateContent, toggleEntry } =
  useFiles();
const { outputs, entryNames, diagnostics, loading, ready, bundleTime, bundle } = useTypack();

useTheme();
useUrlState(files, activeFile);

const MQ = "(max-width: 767px)";
const isMobile = ref(window.matchMedia(MQ).matches);
const mobilePanel = ref<"editor" | "output">("editor");

let mql: MediaQueryList | null = null;

function onMediaChange(e: MediaQueryListEvent) {
  isMobile.value = e.matches;
}

onMounted(() => {
  mql = window.matchMedia(MQ);
  mql.addEventListener("change", onMediaChange);
});

onBeforeUnmount(() => {
  mql?.removeEventListener("change", onMediaChange);
});

let debounceTimer: ReturnType<typeof setTimeout> | undefined;

watch(
  files,
  () => {
    if (!ready.value) return;
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      const fileMap: Record<string, string> = {};
      for (const f of files.value) {
        fileMap[f.name] = f.content;
      }
      const entries = files.value.filter((f) => f.isEntry).map((f) => f.name);
      bundle(fileMap, entries);
    }, 300);
  },
  { deep: true },
);

watch(ready, (isReady) => {
  if (isReady) {
    const fileMap: Record<string, string> = {};
    for (const f of files.value) {
      fileMap[f.name] = f.content;
    }
    const entries = files.value.filter((f) => f.isEntry).map((f) => f.name);
    bundle(fileMap, entries);
  }
});
</script>

<template>
  <div class="flex h-full flex-col bg-white dark:bg-neutral-900">
    <HeaderBar
      :loading="loading"
      :ready="ready"
      :bundle-time="bundleTime"
      :files="files"
      :outputs="outputs"
      :entry-names="entryNames"
    />
    <!-- Desktop: side-by-side splitpanes -->
    <Splitpanes v-if="!isMobile" class="default-theme flex-1 overflow-hidden">
      <Pane :size="50" :min-size="20">
        <EditorPanel
          :files="files"
          :active-file="activeFile"
          @update:active-file="activeFile = $event"
          @update:content="updateContent"
          @add-file="addFile"
          @remove-file="removeFile"
          @rename-file="renameFile"
          @toggle-entry="toggleEntry"
        />
      </Pane>
      <Pane :size="50" :min-size="20">
        <OutputPanel :outputs="outputs" :entry-names="entryNames" :diagnostics="diagnostics" />
      </Pane>
    </Splitpanes>

    <!-- Mobile: tabbed panels -->
    <template v-else>
      <div
        class="flex shrink-0 border-b border-slate-300 bg-slate-100 dark:border-neutral-700 dark:bg-neutral-800"
      >
        <button
          class="flex-1 cursor-pointer border-none bg-transparent py-2 text-xs font-medium transition-colors"
          :class="
            mobilePanel === 'editor'
              ? 'border-b-2 border-b-blue-500 text-slate-900 dark:text-white'
              : 'text-slate-500 dark:text-neutral-400'
          "
          @click="mobilePanel = 'editor'"
        >
          Editor
        </button>
        <button
          class="flex-1 cursor-pointer border-none bg-transparent py-2 text-xs font-medium transition-colors"
          :class="
            mobilePanel === 'output'
              ? 'border-b-2 border-b-blue-500 text-slate-900 dark:text-white'
              : 'text-slate-500 dark:text-neutral-400'
          "
          @click="mobilePanel = 'output'"
        >
          Output
        </button>
      </div>
      <div class="flex-1 overflow-hidden">
        <EditorPanel
          v-show="mobilePanel === 'editor'"
          :files="files"
          :active-file="activeFile"
          class="h-full"
          @update:active-file="activeFile = $event"
          @update:content="updateContent"
          @add-file="addFile"
          @remove-file="removeFile"
          @rename-file="renameFile"
          @toggle-entry="toggleEntry"
        />
        <OutputPanel
          v-show="mobilePanel === 'output'"
          :outputs="outputs"
          :entry-names="entryNames"
          :diagnostics="diagnostics"
          class="h-full"
        />
      </div>
    </template>
  </div>
</template>
