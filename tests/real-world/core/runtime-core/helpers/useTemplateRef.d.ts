import { type ShallowRef } from '@vue/reactivity';
import { type Data } from '../component';
export declare const knownTemplateRefs: WeakSet<ShallowRef>;
export type TemplateRef<T = unknown> = Readonly<ShallowRef<T | null>>;
export declare function useTemplateRef<T = unknown, Keys extends string = string>(key: Keys): TemplateRef<T>;
export declare function isTemplateRefKey(refs: Data, key: string): boolean;
