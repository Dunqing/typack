import { type TrackOpTypes, TriggerOpTypes } from './constants';
/**
 * Incremented every time a reactive change happens
 * This is used to give computed a fast path to avoid re-compute when nothing
 * has changed.
 */
export declare let globalVersion: number;
type KeyToDepMap = Map<any, Dep>;
export declare const targetMap: WeakMap<object, KeyToDepMap>;
export declare const ITERATE_KEY: unique symbol;
export declare const MAP_KEY_ITERATE_KEY: unique symbol;
export declare const ARRAY_ITERATE_KEY: unique symbol;
/**
 * Tracks access to a reactive property.
 *
 * This will check which effect is running at the moment and record it as dep
 * which records all effects that depend on the reactive property.
 *
 * @param target - Object holding the reactive property.
 * @param type - Defines the type of access to the reactive property.
 * @param key - Identifier of the reactive property to track.
 */
export declare function track(target: object, type: TrackOpTypes, key: unknown): void;
/**
 * Finds all deps associated with the target (or a specific property) and
 * triggers the effects stored within.
 *
 * @param target - The reactive object.
 * @param type - Defines the type of the operation that needs to trigger effects.
 * @param key - Can be used to target a specific reactive property in the target object.
 */
export declare function trigger(target: object, type: TriggerOpTypes, key?: unknown, newValue?: unknown, oldValue?: unknown, oldTarget?: Map<unknown, unknown> | Set<unknown>): void;
export declare function getDepFromReactive(object: any, key: string | number | symbol): Dep | undefined;
export {};
