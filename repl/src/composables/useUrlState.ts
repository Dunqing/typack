import pako from "pako";
import { watch, type Ref } from "vue";

import type { FileEntry } from "./useFiles";

interface SerializedState {
  files: FileEntry[];
  active: string;
}

function uint8ToBase64(bytes: Uint8Array): string {
  const chunks: string[] = [];
  for (let i = 0; i < bytes.length; i += 0x8000) {
    const slice = bytes.subarray(i, i + 0x8000);
    let str = "";
    for (let j = 0; j < slice.length; j++) {
      str += String.fromCharCode(slice[j]);
    }
    chunks.push(str);
  }
  return btoa(chunks.join(""));
}

function compress(data: string): string {
  try {
    const bytes = pako.deflate(new TextEncoder().encode(data));
    return uint8ToBase64(bytes);
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
        // Ensure isEntry is defined on all files (backward compat with old URLs)
        const allIsEntryUndefined = state.files.every((f) => f.isEntry === undefined);
        if (allIsEntryUndefined) {
          const indexFile = state.files.find((f) => f.name === "index.d.ts");
          if (indexFile) {
            indexFile.isEntry = true;
          } else {
            state.files[0].isEntry = true;
          }
        }
        // Normalize any remaining undefined isEntry flags to false
        for (const f of state.files) {
          if (f.isEntry === undefined) f.isEntry = false;
        }
        // Ensure at least one file is marked as an entry point
        if (!state.files.some((f) => f.isEntry)) {
          const indexFile = state.files.find((f) => f.name === "index.d.ts");
          if (indexFile) {
            indexFile.isEntry = true;
          } else {
            state.files[0].isEntry = true;
          }
        }
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
