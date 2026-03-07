<script setup lang="ts">
import loader from "@monaco-editor/loader";
import { ref, watch, onMounted, onBeforeUnmount, nextTick, computed } from "vue";

import type { FileEntry } from "../composables/useFiles";
import { useTheme } from "../composables/useTheme";

const { monacoTheme } = useTheme();

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
let linkProviderDisposable: any = null;

onMounted(async () => {
  monaco = await loader.init();
  if (!editorContainer.value) return;

  monaco.languages.typescript.typescriptDefaults.setCompilerOptions({
    target: monaco.languages.typescript.ScriptTarget.ESNext,
    module: monaco.languages.typescript.ModuleKind.ESNext,
    moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
    declaration: true,
    allowImportingTsExtensions: true,
    noEmit: true,
  });
  monaco.languages.typescript.typescriptDefaults.setDiagnosticsOptions({
    noSemanticValidation: true,
  });
  // Disable TS worker features (hover, completions, etc.) to prevent
  // "Could not find source file: inmemory://model" errors.
  // Syntax highlighting still works via Monaco's monarch tokenizer.
  monaco.languages.typescript.typescriptDefaults.setModeConfiguration({
    completionItems: false,
    hovers: false,
    definitions: false,
    references: false,
    documentHighlights: false,
    rename: false,
    diagnostics: false,
    codeActions: false,
    inlayHints: false,
    signatureHelp: false,
    documentSymbols: false,
  });

  // Make import paths clickable — clicking navigates to the target file
  linkProviderDisposable = monaco.languages.registerLinkProvider("typescript", {
    provideLinks(model: any) {
      const links: any[] = [];
      for (let i = 1; i <= model.getLineCount(); i++) {
        const line = model.getLineContent(i);
        const re = /(?:from|import)\s+(['"])(\.[^'"]+)\1/g;
        let m;
        while ((m = re.exec(line)) !== null) {
          const specifier = m[2];
          const bare = specifier.replace(/^\.\//, "");
          const candidates = [bare, bare + ".d.ts", bare + ".ts"];
          const target = candidates.find((c) => props.files.some((f) => f.name === c));
          if (!target) continue;

          const specStart = m.index + m[0].indexOf(m[2]) + 1;
          const specEnd = specStart + m[2].length;
          links.push({
            range: {
              startLineNumber: i,
              startColumn: specStart,
              endLineNumber: i,
              endColumn: specEnd,
            },
            tooltip: `Go to ${target}`,
            data: target,
          });
        }
      }
      return { links };
    },
    resolveLink(link: any) {
      emit("update:active-file", link.data);
      return { ...link, url: monaco.Uri.parse(`file:///${link.data}`) };
    },
  });

  // Create models for all files
  for (const file of props.files) {
    const uri = monaco.Uri.parse(`file:///${file.name}`);
    if (!monaco.editor.getModel(uri)) {
      monaco.editor.createModel(file.content, "typescript", uri);
    }
  }

  const activeUri = monaco.Uri.parse(`file:///${props.activeFile}`);
  editor = monaco.editor.create(editorContainer.value, {
    model: monaco.editor.getModel(activeUri),
    theme: monacoTheme.value,
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
  linkProviderDisposable?.dispose();
  editor?.dispose();
  // Dispose all file models
  if (monaco) {
    for (const model of monaco.editor.getModels()) {
      model.dispose();
    }
  }
});

// Watch for theme changes and update Monaco
watch(monacoTheme, (theme) => {
  monaco?.editor.setTheme(theme);
});

function currentContent(): string {
  const f = props.files.find((f) => f.name === props.activeFile);
  return f?.content ?? "";
}

const fileNames = computed(() => props.files.map((f) => f.name));

// Keep Monaco models in sync with files (add/remove/rename)
watch(fileNames, (newNames, oldNames) => {
  if (!monaco) return;
  const newSet = new Set(newNames);
  const oldSet = new Set(oldNames ?? []);

  // Dispose models for removed files
  for (const name of oldSet) {
    if (!newSet.has(name)) {
      const uri = monaco.Uri.parse(`file:///${name}`);
      monaco.editor.getModel(uri)?.dispose();
    }
  }

  // Create models for new files
  for (const file of props.files) {
    if (!oldSet.has(file.name)) {
      const uri = monaco.Uri.parse(`file:///${file.name}`);
      if (!monaco.editor.getModel(uri)) {
        monaco.editor.createModel(file.content, "typescript", uri);
      }
    }
  }
});

watch(
  () => props.activeFile,
  () => {
    nextTick(() => {
      if (editor && monaco) {
        const uri = monaco.Uri.parse(`file:///${props.activeFile}`);
        let model = monaco.editor.getModel(uri);
        if (!model) {
          model = monaco.editor.createModel(currentContent(), "typescript", uri);
        }
        editor.setModel(model);
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
  <div class="flex h-full flex-col">
    <div
      class="flex shrink-0 overflow-x-auto border-b border-slate-300 bg-slate-100 dark:border-neutral-700 dark:bg-neutral-900"
    >
      <div
        v-for="file in files"
        :key="file.name"
        class="flex cursor-pointer items-center gap-1.5 border-r border-slate-300 px-3 py-1.5 text-xs whitespace-nowrap dark:border-neutral-700"
        :class="
          file.name === activeFile
            ? 'border-b-2 border-b-blue-500 bg-white text-slate-900 dark:bg-neutral-900 dark:text-white'
            : 'bg-slate-200 text-slate-500 dark:bg-neutral-800 dark:text-neutral-500'
        "
        @click="emit('update:active-file', file.name)"
        @dblclick="startRename(file.name)"
      >
        <template v-if="editingTab === file.name">
          <input
            v-model="editInput"
            class="w-30 border border-blue-500 bg-slate-200 px-1 py-px text-xs text-slate-900 outline-none dark:bg-neutral-700 dark:text-white"
            @blur="finishRename(file.name)"
            @keyup.enter="finishRename(file.name)"
            @keyup.escape="editingTab = null"
            @click.stop
            autofocus
          />
        </template>
        <template v-else>
          <span>{{ file.name }}</span>
          <button
            v-if="files.length > 1"
            class="cursor-pointer border-none bg-transparent px-0.5 text-sm leading-none text-slate-400 hover:text-slate-900 dark:text-neutral-600 dark:hover:text-white"
            @click.stop="emit('remove-file', file.name)"
          >
            &times;
          </button>
        </template>
      </div>
      <button
        class="cursor-pointer border-none bg-transparent px-3 py-1.5 text-base text-slate-400 hover:text-slate-900 dark:text-neutral-600 dark:hover:text-white"
        @click="emit('add-file')"
      >
        +
      </button>
    </div>
    <div ref="editorContainer" class="flex-1" />
  </div>
</template>
