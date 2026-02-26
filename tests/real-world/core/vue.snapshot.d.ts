import { CompilerOptions } from "@vue/compiler-dom";
import { RenderFunction } from "@vue/runtime-dom";
export * from "@vue/runtime-dom";

//#region tests/real-world/core/vue.d.ts
declare function compileToFunction(template: string | HTMLElement, options?: CompilerOptions): RenderFunction;
//#endregion
export { compileToFunction as compile };