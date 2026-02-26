import { type CompilerOptions } from '@vue/compiler-dom';
import { type RenderFunction } from '@vue/runtime-dom';
declare function compileToFunction(template: string | HTMLElement, options?: CompilerOptions): RenderFunction;
export { compileToFunction as compile };
export * from '@vue/runtime-dom';
