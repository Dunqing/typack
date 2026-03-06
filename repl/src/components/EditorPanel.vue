<script setup lang="ts">
import loader from "@monaco-editor/loader";
import { ref, watch, onMounted, onBeforeUnmount, nextTick } from "vue";

import type { FileEntry } from "../composables/useFiles";

const props = defineProps<{
  files: FileEntry[];
  activeFile: string;
}>();

const emit = defineEmits<{
  "update:active-file": [name: string];
  "update:content": [payload: { name: string; content: string }];
  "add-file": [];
  "remove-file": [name: string];
  "rename-file": [payload: { oldName: string; newName: string }];
}>();

const editorContainer = ref<HTMLDivElement>();
let editor: any = null;
let monaco: any = null;

onMounted(async () => {
  monaco = await loader.init();
  if (!editorContainer.value) return;
  editor = monaco.editor.create(editorContainer.value, {
    value: currentContent(),
    language: "typescript",
    theme: "vs-dark",
    minimap: { enabled: false },
    fontSize: 13,
    lineNumbers: "on",
    scrollBeyondLastLine: false,
    automaticLayout: true,
    tabSize: 2,
  });
  editor.onDidChangeModelContent(() => {
    emit("update:content", {
      name: props.activeFile,
      content: editor.getValue(),
    });
  });
});

onBeforeUnmount(() => {
  editor?.dispose();
});

function currentContent(): string {
  const f = props.files.find((f) => f.name === props.activeFile);
  return f?.content ?? "";
}

watch(
  () => props.activeFile,
  () => {
    nextTick(() => {
      if (editor) {
        const val = currentContent();
        if (editor.getValue() !== val) {
          editor.setValue(val);
        }
      }
    });
  },
);

const editingTab = ref<string | null>(null);
const editInput = ref("");

function startRename(name: string) {
  editingTab.value = name;
  editInput.value = name;
}

function finishRename(oldName: string) {
  const newName = editInput.value.trim();
  editingTab.value = null;
  if (newName && newName !== oldName) {
    emit("rename-file", { oldName, newName });
  }
}
</script>

<template>
  <div class="editor-panel">
    <div class="tabs">
      <div
        v-for="file in files"
        :key="file.name"
        class="tab"
        :class="{ active: file.name === activeFile }"
        @click="emit('update:active-file', file.name)"
        @dblclick="startRename(file.name)"
      >
        <template v-if="editingTab === file.name">
          <input
            v-model="editInput"
            class="tab-input"
            @blur="finishRename(file.name)"
            @keyup.enter="finishRename(file.name)"
            @keyup.escape="editingTab = null"
            @click.stop
            autofocus
          />
        </template>
        <template v-else>
          <span class="tab-name">{{ file.name }}</span>
          <button
            v-if="files.length > 1"
            class="tab-close"
            @click.stop="emit('remove-file', file.name)"
          >
            &times;
          </button>
        </template>
      </div>
      <button class="tab add-tab" @click="emit('add-file')">+</button>
    </div>
    <div ref="editorContainer" class="editor-container" />
  </div>
</template>

<style scoped>
.editor-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}
.tabs {
  display: flex;
  background: #1e1e1e;
  border-bottom: 1px solid #333;
  overflow-x: auto;
  flex-shrink: 0;
}
.tab {
  display: flex;
  align-items: center;
  padding: 6px 12px;
  color: #999;
  cursor: pointer;
  font-size: 12px;
  border-right: 1px solid #333;
  white-space: nowrap;
  gap: 6px;
}
.tab.active {
  color: #fff;
  background: #1e1e1e;
  border-bottom: 2px solid #3b82f6;
}
.tab:not(.active) {
  background: #2d2d2d;
}
.tab-close {
  background: none;
  border: none;
  color: #666;
  cursor: pointer;
  font-size: 14px;
  padding: 0 2px;
  line-height: 1;
}
.tab-close:hover {
  color: #fff;
}
.add-tab {
  color: #666;
  font-size: 16px;
  border: none;
  background: none;
  padding: 6px 12px;
  cursor: pointer;
}
.add-tab:hover {
  color: #fff;
}
.tab-input {
  background: #333;
  border: 1px solid #3b82f6;
  color: #fff;
  font-size: 12px;
  padding: 1px 4px;
  width: 120px;
  outline: none;
}
.editor-container {
  flex: 1;
}
</style>
