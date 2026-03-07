<script setup lang="ts">
import type { FileEntry } from "../composables/useFiles";
import { useTheme } from "../composables/useTheme";

const { isDark, toggle } = useTheme();

const props = defineProps<{
  loading: boolean;
  ready: boolean;
  bundleTime: number;
  files: FileEntry[];
  output: string;
}>();

function reportBug() {
  const input = props.files.map((f) => `// ${f.name}\n${f.content}`).join("\n\n");

  const replLink = window.location.href;

  const params = new URLSearchParams({
    template: "bug_report.yml",
    title: "[Bug] ",
    input: input,
    output: props.output,
    "repl-link": replLink,
  });

  window.open(
    `https://github.com/Dunqing/typack/issues/new?${params.toString()}`,
    "_blank",
    "noopener,noreferrer",
  );
}
</script>

<template>
  <header
    class="flex h-12 shrink-0 items-center justify-between border-b border-slate-200 bg-white px-3 py-2 text-slate-800 sm:px-4 dark:border-neutral-700 dark:bg-neutral-900 dark:text-slate-100"
  >
    <div class="flex min-w-0 items-center gap-2 sm:gap-3">
      <h1 class="shrink-0 text-sm font-semibold sm:text-base">Typack REPL</h1>
      <span v-if="!ready" class="rounded bg-amber-400 px-2 py-0.5 text-xs text-black"
        >Loading WASM...</span
      >
      <span v-else-if="loading" class="rounded bg-blue-500 px-2 py-0.5 text-xs text-white"
        >Bundling...</span
      >
      <span v-else class="rounded bg-green-500 px-2 py-0.5 text-xs text-black">
        Ready<template v-if="bundleTime > 0"> · {{ bundleTime }}ms</template>
      </span>
    </div>
    <div class="flex items-center gap-2 sm:gap-3">
      <button
        type="button"
        class="flex cursor-pointer items-center gap-1.5 rounded-md border border-slate-300 bg-transparent px-2 py-1 text-xs text-inherit transition-colors hover:border-slate-400 hover:bg-slate-200 sm:px-2.5 dark:border-slate-600 dark:hover:border-slate-500 dark:hover:bg-slate-700"
        @click="reportBug"
        title="Report Bug"
      >
        <svg viewBox="0 0 16 16" width="16" height="16" fill="currentColor">
          <path
            d="M4.72.22a.75.75 0 0 1 1.06 0l1 .999a3.49 3.49 0 0 1 2.441 0l.999-1a.75.75 0 1 1
            1.06 1.062l-.69.691a3.503 3.503 0 0 1 1.39 2.217l1.782-.884a.75.75 0 0 1
            .67 1.34L12.64 5.53l.001.074a3.5 3.5 0 0 1-.022.427h2.131a.75.75 0 0 1
            0 1.5h-2.318a3.5 3.5 0 0 1-1.207 1.678l1.463 1.462a.75.75 0 0 1-1.06
            1.06l-1.586-1.585a3.5 3.5 0 0 1-1.085.2v3.404a.75.75 0 0 1-1.5
            0v-3.404a3.5 3.5 0 0 1-1.085-.2L5.787 11.73a.75.75 0 0 1-1.06-1.06l1.462-1.462a3.5
            3.5 0 0 1-1.207-1.678H2.75a.75.75 0 0 1 0-1.5h2.132a3.5 3.5 0 0 1-.023-.426l.001-.075
            L3.067 4.646a.75.75 0 0 1 .67-1.34l1.783.884A3.503 3.503 0 0 1 6.93 1.97l-.69-.69A.75.75
            0 0 1 4.72.22ZM6.173 5.98a2 2 0 1 0 3.654 0H6.173Z"
          />
        </svg>
        <span class="hidden sm:inline">Report Bug</span>
      </button>
      <button
        type="button"
        class="flex cursor-pointer items-center opacity-70 transition-opacity hover:opacity-100"
        @click="toggle"
        :title="isDark ? 'Switch to light mode' : 'Switch to dark mode'"
      >
        <!-- Sun icon (shown in dark mode) -->
        <svg
          v-if="isDark"
          viewBox="0 0 24 24"
          width="20"
          height="20"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <circle cx="12" cy="12" r="5" />
          <line x1="12" y1="1" x2="12" y2="3" />
          <line x1="12" y1="21" x2="12" y2="23" />
          <line x1="4.22" y1="4.22" x2="5.64" y2="5.64" />
          <line x1="18.36" y1="18.36" x2="19.78" y2="19.78" />
          <line x1="1" y1="12" x2="3" y2="12" />
          <line x1="21" y1="12" x2="23" y2="12" />
          <line x1="4.22" y1="19.78" x2="5.64" y2="18.36" />
          <line x1="18.36" y1="5.64" x2="19.78" y2="4.22" />
        </svg>
        <!-- Moon icon (shown in light mode) -->
        <svg
          v-else
          viewBox="0 0 24 24"
          width="20"
          height="20"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z" />
        </svg>
      </button>
      <a
        href="https://github.com/Dunqing/typack"
        target="_blank"
        rel="noopener noreferrer"
        class="flex items-center opacity-70 transition-opacity hover:opacity-100"
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
