import { type DebuggerOptions, type ReactiveMarker, type WatchCallback, type WatchEffect, type WatchHandle, type WatchSource } from '@vue/reactivity';
import { type ComponentInternalInstance } from './component';
import type { ObjectWatchOptionItem } from './componentOptions';
import type { ComponentPublicInstance } from './componentPublicInstance';
export type { WatchHandle, WatchStopHandle, WatchEffect, WatchSource, WatchCallback, OnCleanup, } from '@vue/reactivity';
type MaybeUndefined<T, I> = I extends true ? T | undefined : T;
type MapSources<T, Immediate> = {
    [K in keyof T]: T[K] extends WatchSource<infer V> ? MaybeUndefined<V, Immediate> : T[K] extends object ? MaybeUndefined<T[K], Immediate> : never;
};
export interface WatchEffectOptions extends DebuggerOptions {
    flush?: 'pre' | 'post' | 'sync';
}
export interface WatchOptions<Immediate = boolean> extends WatchEffectOptions {
    immediate?: Immediate;
    deep?: boolean | number;
    once?: boolean;
}
export declare function watchEffect(effect: WatchEffect, options?: WatchEffectOptions): WatchHandle;
export declare function watchPostEffect(effect: WatchEffect, options?: DebuggerOptions): WatchHandle;
export declare function watchSyncEffect(effect: WatchEffect, options?: DebuggerOptions): WatchHandle;
export type MultiWatchSources = (WatchSource<unknown> | object)[];
export declare function watch<T, Immediate extends Readonly<boolean> = false>(source: WatchSource<T>, cb: WatchCallback<T, MaybeUndefined<T, Immediate>>, options?: WatchOptions<Immediate>): WatchHandle;
export declare function watch<T extends Readonly<MultiWatchSources>, Immediate extends Readonly<boolean> = false>(sources: readonly [...T] | T, cb: [T] extends [ReactiveMarker] ? WatchCallback<T, MaybeUndefined<T, Immediate>> : WatchCallback<MapSources<T, false>, MapSources<T, Immediate>>, options?: WatchOptions<Immediate>): WatchHandle;
export declare function watch<T extends MultiWatchSources, Immediate extends Readonly<boolean> = false>(sources: [...T], cb: WatchCallback<MapSources<T, false>, MapSources<T, Immediate>>, options?: WatchOptions<Immediate>): WatchHandle;
export declare function watch<T extends object, Immediate extends Readonly<boolean> = false>(source: T, cb: WatchCallback<T, MaybeUndefined<T, Immediate>>, options?: WatchOptions<Immediate>): WatchHandle;
export declare function instanceWatch(this: ComponentInternalInstance, source: string | Function, value: WatchCallback | ObjectWatchOptionItem, options?: WatchOptions): WatchHandle;
export declare function createPathGetter(ctx: ComponentPublicInstance, path: string): () => WatchSource | WatchSource[] | WatchEffect | object;
