import { CodegenResult, CompilerOptions, RootNode } from "@vue/compiler-dom";

//#region tests/real-world/core/compiler-ssr.d.ts
export declare function compile(source: string | RootNode, options?: CompilerOptions): CodegenResult;
//#endregion