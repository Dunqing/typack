import pako from "pako";
import { watch, type Ref } from "vue";

import type { FileEntry } from "./useFiles";

interface SerializedState {
  files: FileEntry[];
  active: string;
}

function compress(data: string): string {
  try {
    const bytes = pako.deflate(new TextEncoder().encode(data));
    return btoa(String.fromCharCode(...bytes));
  } catch {
    return btoa(encodeURIComponent(data));
  }
}

function decompress(encoded: string): string {
  try {
    const bytes = Uint8Array.from(atob(encoded), (c) => c.charCodeAt(0));
    return new TextDecoder().decode(pako.inflate(bytes));
  } catch {
    try {
      return decodeURIComponent(atob(encoded));
    } catch {
      return "";
    }
  }
}

export function useUrlState(files: Ref<FileEntry[]>, activeFile: Ref<string>) {
  // Restore from URL hash
  const hash = location.hash.slice(1);
  if (hash) {
    try {
      const json = decompress(hash);
      const state: SerializedState = JSON.parse(json);
      if (state.files?.length) {
        files.value = state.files;
        activeFile.value = state.active || state.files[0].name;
      }
    } catch {
      // Invalid hash, ignore
    }
  }

  // Save to URL hash (debounced)
  let timer: ReturnType<typeof setTimeout> | undefined;
  watch(
    [files, activeFile],
    () => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        const state: SerializedState = {
          files: files.value,
          active: activeFile.value,
        };
        const encoded = compress(JSON.stringify(state));
        history.replaceState(null, "", `#${encoded}`);
      }, 500);
    },
    { deep: true },
  );
}
