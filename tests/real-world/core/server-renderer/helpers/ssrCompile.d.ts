import { type ComponentInternalInstance } from 'vue';
import type { PushFn } from '../render';
type SSRRenderFunction = (context: any, push: PushFn, parentInstance: ComponentInternalInstance) => void;
export declare function ssrCompile(template: string, instance: ComponentInternalInstance): SSRRenderFunction;
export {};
