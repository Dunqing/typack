import type { DefineModelOptions, ModelRef } from '../apiSetupHelpers';
export declare function useModel<M extends PropertyKey, T extends Record<string, any>, K extends keyof T, G = T[K], S = T[K]>(props: T, name: K, options?: DefineModelOptions<T[K], G, S>): ModelRef<T[K], M, G, S>;
export declare const getModelModifiers: (props: Record<string, any>, modelName: string) => Record<string, boolean> | undefined;
