import { ref, type Ref } from "vue";

export interface FileEntry {
  name: string;
  content: string;
}

const DEFAULT_FILES: FileEntry[] = [
  {
    name: "index.d.ts",
    content: `import { add } from './utils';\n\nexport declare function greet(name: string): string;\nexport { add };\n`,
  },
  {
    name: "utils.d.ts",
    content: `export declare function add(a: number, b: number): number;\nexport declare function subtract(a: number, b: number): number;\n`,
  },
];

export function useFiles(initial?: FileEntry[]) {
  const files: Ref<FileEntry[]> = ref(initial ?? DEFAULT_FILES.map((f) => ({ ...f })));
  const activeFile = ref(files.value[0]?.name ?? "");

  function addFile() {
    let i = 1;
    let name = `file${i}.d.ts`;
    while (files.value.some((f) => f.name === name)) {
      i++;
      name = `file${i}.d.ts`;
    }
    files.value.push({ name, content: "" });
    activeFile.value = name;
  }

  function removeFile(name: string) {
    const idx = files.value.findIndex((f) => f.name === name);
    if (idx === -1 || files.value.length <= 1) return;
    files.value.splice(idx, 1);
    if (activeFile.value === name) {
      activeFile.value = files.value[Math.min(idx, files.value.length - 1)].name;
    }
  }

  function renameFile({ oldName, newName }: { oldName: string; newName: string }) {
    const file = files.value.find((f) => f.name === oldName);
    if (!file) return;
    if (files.value.some((f) => f.name === newName)) return;
    file.name = newName;
    if (activeFile.value === oldName) {
      activeFile.value = newName;
    }
  }

  function updateContent({ name, content }: { name: string; content: string }) {
    const file = files.value.find((f) => f.name === name);
    if (file) file.content = content;
  }

  return { files, activeFile, addFile, removeFile, renameFile, updateContent };
}
