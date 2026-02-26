import type { ComputedRef } from './computed';
import { type DebuggerOptions, ReactiveEffect } from './effect';
import { type Ref } from './ref';
export declare enum WatchErrorCodes {
    WATCH_GETTER = 2,
    WATCH_CALLBACK = 3,
    WATCH_CLEANUP = 4
}
export type WatchEffect = (onCleanup: OnCleanup) => void;
export type WatchSource<T = any> = Ref<T, any> | ComputedRef<T> | (() => T);
export type WatchCallback<V = any, OV = any> = (value: V, oldValue: OV, onCleanup: OnCleanup) => any;
export type OnCleanup = (cleanupFn: () => void) => void;
export interface WatchOptions<Immediate = boolean> extends DebuggerOptions {
    immediate?: Immediate;
    deep?: boolean | number;
    once?: boolean;
    scheduler?: WatchScheduler;
    onWarn?: (msg: string, ...args: any[]) => void;
}
export type WatchStopHandle = () => void;
export interface WatchHandle extends WatchStopHandle {
    pause: () => void;
    resume: () => void;
    stop: () => void;
}
export type WatchScheduler = (job: () => void, isFirstRun: boolean) => void;
/**
 * Returns the current active effect if there is one.
 */
export declare function getCurrentWatcher(): ReactiveEffect<any> | undefined;
/**
 * Registers a cleanup callback on the current active effect. This
 * registered cleanup callback will be invoked right before the
 * associated effect re-runs.
 *
 * @param cleanupFn - The callback function to attach to the effect's cleanup.
 * @param failSilently - if `true`, will not throw warning when called without
 * an active effect.
 * @param owner - The effect that this cleanup function should be attached to.
 * By default, the current active effect.
 */
export declare function onWatcherCleanup(cleanupFn: () => void, failSilently?: boolean, owner?: ReactiveEffect | undefined): void;
export declare function watch(source: WatchSource | WatchSource[] | WatchEffect | object, cb?: WatchCallback | null, options?: WatchOptions): WatchHandle;
export declare function traverse(value: unknown, depth?: number, seen?: Map<unknown, number>): unknown;
