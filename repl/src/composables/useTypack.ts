import { ref, shallowRef } from "vue";

interface BundleResult {
  code: string;
  map: string | null;
}

interface Diagnostic {
  message: string;
  severity: string;
}

interface TypackModule {
  bundle: (options: {
    input: string[];
    cwd?: string;
    sourcemap?: boolean;
    cjsDefault?: boolean;
    external?: string[];
  }) => BundleResult & { warnings: Diagnostic[] };
}

export function useTypack() {
  const output = shallowRef<BundleResult>({ code: "", map: null });
  const diagnostics = ref<Diagnostic[]>([]);
  const loading = ref(false);
  const ready = ref(false);

  let typackModule: TypackModule | null = null;
  let vol: any = null;

  async function init() {
    try {
      const { Volume, createFsFromVolume } = await import("@napi-rs/wasm-runtime/fs");
      const { instantiateNapiModuleSync, getDefaultContext, WASI } =
        await import("@napi-rs/wasm-runtime");

      vol = new Volume();
      const fs = createFsFromVolume(vol);
      // Ensure root dir exists
      try {
        fs.mkdirSync("/src", { recursive: true });
      } catch {}

      const wasi = new WASI({
        version: "preview1" as any,
        fs,
        preopens: { "/": "/" },
      });

      const base = import.meta.env.BASE_URL || "/";
      const wasmUrl = `${base}wasm/typack.wasm`;
      const wasmFile = await fetch(wasmUrl).then((r) => r.arrayBuffer());

      const sharedMemory = new WebAssembly.Memory({
        initial: 4000,
        maximum: 65536,
        shared: true,
      });

      const { napiModule } = instantiateNapiModuleSync(wasmFile, {
        context: getDefaultContext(),
        asyncWorkPoolSize: 0,
        wasi,
        overwriteImports(importObject: any) {
          importObject.env = {
            ...importObject.env,
            ...importObject.napi,
            ...importObject.emnapi,
            memory: sharedMemory,
          };
          return importObject;
        },
        beforeInit({ instance }: any) {
          for (const name of Object.keys(instance.exports)) {
            if (name.startsWith("__napi_register__")) {
              (instance.exports as any)[name]();
            }
          }
        },
      });

      typackModule = napiModule.exports as unknown as TypackModule;
      ready.value = true;
    } catch (err) {
      console.error("Failed to load typack WASM:", err);
      diagnostics.value = [{ message: `Failed to load WASM: ${err}`, severity: "error" }];
    }
  }

  init();

  function bundle(files: Record<string, string>) {
    if (!typackModule || !vol) return;

    loading.value = true;
    diagnostics.value = [];

    try {
      // Clear and repopulate the virtual filesystem
      vol.reset();
      try {
        vol.mkdirSync("/src", { recursive: true });
      } catch {}
      for (const [name, content] of Object.entries(files)) {
        vol.writeFileSync(`/src/${name}`, content, "utf8");
      }

      const result = typackModule.bundle({
        input: ["/src/index.d.ts"],
        cwd: "/src",
        sourcemap: true,
      });

      output.value = { code: result.code, map: result.map ?? null };
      diagnostics.value = result.warnings ?? [];
    } catch (err: any) {
      // NAPI errors encode diagnostics as JSON in the message
      let errors: Diagnostic[];
      try {
        errors = JSON.parse(err.message);
      } catch {
        errors = [{ message: String(err.message ?? err), severity: "error" }];
      }
      diagnostics.value = errors;
      output.value = { code: "", map: null };
    } finally {
      loading.value = false;
    }
  }

  return { output, diagnostics, loading, ready, bundle };
}
