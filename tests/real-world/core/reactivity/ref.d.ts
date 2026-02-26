import { type IfAny } from '@vue/shared';
import { type Builtin, type ShallowReactiveMarker } from './reactive';
import type { ComputedRef, WritableComputedRef } from './computed';
declare const RefSymbol: unique symbol;
export declare const RawSymbol: unique symbol;
export interface Ref<T = any, S = T> {
    get value(): T;
    set value(_: S);
    /**
     * Type differentiator only.
     * We need this to be in public d.ts but don't want it to show up in IDE
     * autocomplete, so we use a private Symbol instead.
     */
    [RefSymbol]: true;
}
/**
 * Checks if a value is a ref object.
 *
 * @param r - The value to inspect.
 * @see {@link https://vuejs.org/api/reactivity-utilities.html#isref}
 */
export declare function isRef<T>(r: Ref<T> | unknown): r is Ref<T>;
/**
 * Takes an inner value and returns a reactive and mutable ref object, which
 * has a single property `.value` that points to the inner value.
 *
 * @param value - The object to wrap in the ref.
 * @see {@link https://vuejs.org/api/reactivity-core.html#ref}
 */
export declare function ref<T>(value: T): [T] extends [Ref] ? IfAny<T, Ref<T>, T> : Ref<UnwrapRef<T>, UnwrapRef<T> | T>;
export declare function ref<T = any>(): Ref<T | undefined>;
declare const ShallowRefMarker: unique symbol;
export type ShallowRef<T = any, S = T> = Ref<T, S> & {
    [ShallowRefMarker]?: true;
};
/**
 * Shallow version of {@link ref}.
 *
 * @example
 * ```js
 * const state = shallowRef({ count: 1 })
 *
 * // does NOT trigger change
 * state.value.count = 2
 *
 * // does trigger change
 * state.value = { count: 2 }
 * ```
 *
 * @param value - The "inner value" for the shallow ref.
 * @see {@link https://vuejs.org/api/reactivity-advanced.html#shallowref}
 */
export declare function shallowRef<T>(value: T): Ref extends T ? T extends Ref ? IfAny<T, ShallowRef<T>, T> : ShallowRef<T> : ShallowRef<T>;
export declare function shallowRef<T = any>(): ShallowRef<T | undefined>;
/**
 * Force trigger effects that depends on a shallow ref. This is typically used
 * after making deep mutations to the inner value of a shallow ref.
 *
 * @example
 * ```js
 * const shallow = shallowRef({
 *   greet: 'Hello, world'
 * })
 *
 * // Logs "Hello, world" once for the first run-through
 * watchEffect(() => {
 *   console.log(shallow.value.greet)
 * })
 *
 * // This won't trigger the effect because the ref is shallow
 * shallow.value.greet = 'Hello, universe'
 *
 * // Logs "Hello, universe"
 * triggerRef(shallow)
 * ```
 *
 * @param ref - The ref whose tied effects shall be executed.
 * @see {@link https://vuejs.org/api/reactivity-advanced.html#triggerref}
 */
export declare function triggerRef(ref: Ref): void;
export type MaybeRef<T = any> = T | Ref<T> | ShallowRef<T> | WritableComputedRef<T>;
export type MaybeRefOrGetter<T = any> = MaybeRef<T> | ComputedRef<T> | (() => T);
/**
 * Returns the inner value if the argument is a ref, otherwise return the
 * argument itself. This is a sugar function for
 * `val = isRef(val) ? val.value : val`.
 *
 * @example
 * ```js
 * function useFoo(x: number | Ref<number>) {
 *   const unwrapped = unref(x)
 *   // unwrapped is guaranteed to be number now
 * }
 * ```
 *
 * @param ref - Ref or plain value to be converted into the plain value.
 * @see {@link https://vuejs.org/api/reactivity-utilities.html#unref}
 */
export declare function unref<T>(ref: MaybeRef<T> | ComputedRef<T>): T;
/**
 * Normalizes values / refs / getters to values.
 * This is similar to {@link unref}, except that it also normalizes getters.
 * If the argument is a getter, it will be invoked and its return value will
 * be returned.
 *
 * @example
 * ```js
 * toValue(1) // 1
 * toValue(ref(1)) // 1
 * toValue(() => 1) // 1
 * ```
 *
 * @param source - A getter, an existing ref, or a non-function value.
 * @see {@link https://vuejs.org/api/reactivity-utilities.html#tovalue}
 */
export declare function toValue<T>(source: MaybeRefOrGetter<T>): T;
/**
 * Returns a proxy for the given object that shallowly unwraps properties that
 * are refs. If the object already is reactive, it's returned as-is. If not, a
 * new reactive proxy is created.
 *
 * @param objectWithRefs - Either an already-reactive object or a simple object
 * that contains refs.
 */
export declare function proxyRefs<T extends object>(objectWithRefs: T): ShallowUnwrapRef<T>;
export type CustomRefFactory<T> = (track: () => void, trigger: () => void) => {
    get: () => T;
    set: (value: T) => void;
};
/**
 * Creates a customized ref with explicit control over its dependency tracking
 * and updates triggering.
 *
 * @param factory - The function that receives the `track` and `trigger` callbacks.
 * @see {@link https://vuejs.org/api/reactivity-advanced.html#customref}
 */
export declare function customRef<T>(factory: CustomRefFactory<T>): Ref<T>;
export type ToRefs<T = any> = {
    [K in keyof T]: ToRef<T[K]>;
};
/**
 * Converts a reactive object to a plain object where each property of the
 * resulting object is a ref pointing to the corresponding property of the
 * original object. Each individual ref is created using {@link toRef}.
 *
 * @param object - Reactive object to be made into an object of linked refs.
 * @see {@link https://vuejs.org/api/reactivity-utilities.html#torefs}
 */
export declare function toRefs<T extends object>(object: T): ToRefs<T>;
export type ToRef<T> = IfAny<T, Ref<T>, [T] extends [Ref] ? T : Ref<T>>;
/**
 * Used to normalize values / refs / getters into refs.
 *
 * @example
 * ```js
 * // returns existing refs as-is
 * toRef(existingRef)
 *
 * // creates a ref that calls the getter on .value access
 * toRef(() => props.foo)
 *
 * // creates normal refs from non-function values
 * // equivalent to ref(1)
 * toRef(1)
 * ```
 *
 * Can also be used to create a ref for a property on a source reactive object.
 * The created ref is synced with its source property: mutating the source
 * property will update the ref, and vice-versa.
 *
 * @example
 * ```js
 * const state = reactive({
 *   foo: 1,
 *   bar: 2
 * })
 *
 * const fooRef = toRef(state, 'foo')
 *
 * // mutating the ref updates the original
 * fooRef.value++
 * console.log(state.foo) // 2
 *
 * // mutating the original also updates the ref
 * state.foo++
 * console.log(fooRef.value) // 3
 * ```
 *
 * @param source - A getter, an existing ref, a non-function value, or a
 *                 reactive object to create a property ref from.
 * @param [key] - (optional) Name of the property in the reactive object.
 * @see {@link https://vuejs.org/api/reactivity-utilities.html#toref}
 */
export declare function toRef<T>(value: T): T extends () => infer R ? Readonly<Ref<R>> : T extends Ref ? T : Ref<UnwrapRef<T>>;
export declare function toRef<T extends object, K extends keyof T>(object: T, key: K): ToRef<T[K]>;
export declare function toRef<T extends object, K extends keyof T>(object: T, key: K, defaultValue: T[K]): ToRef<Exclude<T[K], undefined>>;
/**
 * This is a special exported interface for other packages to declare
 * additional types that should bail out for ref unwrapping. For example
 * \@vue/runtime-dom can declare it like so in its d.ts:
 *
 * ``` ts
 * declare module '@vue/reactivity' {
 *   export interface RefUnwrapBailTypes {
 *     runtimeDOMBailTypes: Node | Window
 *   }
 * }
 * ```
 */
export interface RefUnwrapBailTypes {
}
export type ShallowUnwrapRef<T> = {
    [K in keyof T]: DistributeRef<T[K]>;
};
type DistributeRef<T> = T extends Ref<infer V, unknown> ? V : T;
export type UnwrapRef<T> = T extends ShallowRef<infer V, unknown> ? V : T extends Ref<infer V, unknown> ? UnwrapRefSimple<V> : UnwrapRefSimple<T>;
export type UnwrapRefSimple<T> = T extends Builtin | Ref | RefUnwrapBailTypes[keyof RefUnwrapBailTypes] | {
    [RawSymbol]?: true;
} ? T : T extends Map<infer K, infer V> ? Map<K, UnwrapRefSimple<V>> & UnwrapRef<Omit<T, keyof Map<any, any>>> : T extends WeakMap<infer K, infer V> ? WeakMap<K, UnwrapRefSimple<V>> & UnwrapRef<Omit<T, keyof WeakMap<any, any>>> : T extends Set<infer V> ? Set<UnwrapRefSimple<V>> & UnwrapRef<Omit<T, keyof Set<any>>> : T extends WeakSet<infer V> ? WeakSet<UnwrapRefSimple<V>> & UnwrapRef<Omit<T, keyof WeakSet<any>>> : T extends ReadonlyArray<any> ? {
    [K in keyof T]: UnwrapRefSimple<T[K]>;
} : T extends object & {
    [ShallowReactiveMarker]?: never;
} ? {
    [P in keyof T]: P extends symbol ? T[P] : UnwrapRef<T[P]>;
} : T;
export {};
