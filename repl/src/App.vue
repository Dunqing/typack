<script setup lang="ts">
import { Splitpanes, Pane } from "splitpanes";
import { watch } from "vue";

import "splitpanes/dist/splitpanes.css";
import EditorPanel from "./components/EditorPanel.vue";
import HeaderBar from "./components/HeaderBar.vue";
import OutputPanel from "./components/OutputPanel.vue";
import { useFiles } from "./composables/useFiles";
import { useTypack } from "./composables/useTypack";
import { useUrlState } from "./composables/useUrlState";

const { files, activeFile, addFile, removeFile, renameFile, updateContent } = useFiles();
const { output, diagnostics, loading, ready, bundleTime, bundle } = useTypack();

useUrlState(files, activeFile);

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
      bundle(fileMap);
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
    bundle(fileMap);
  }
});
</script>

<template>
  <div class="app">
    <HeaderBar
      :loading="loading"
      :ready="ready"
      :bundle-time="bundleTime"
      :files="files"
      :output="output.code"
    />
    <Splitpanes class="default-theme main-panes">
      <Pane :size="50" :min-size="20">
        <EditorPanel
          :files="files"
          :active-file="activeFile"
          @update:active-file="activeFile = $event"
          @update:content="updateContent"
          @add-file="addFile"
          @remove-file="removeFile"
          @rename-file="renameFile"
        />
      </Pane>
      <Pane :size="50" :min-size="20">
        <OutputPanel :code="output.code" :map="output.map" :diagnostics="diagnostics" />
      </Pane>
    </Splitpanes>
  </div>
</template>

<style>
.app {
  height: 100%;
  display: flex;
  flex-direction: column;
}
.main-panes {
  flex: 1;
  overflow: hidden;
}
.splitpanes__pane {
  display: flex;
  flex-direction: column;
}
.splitpanes__splitter {
  background: #e2e8f0;
  position: relative;
}
.default-theme .splitpanes__splitter {
  min-width: 4px;
}
</style>
