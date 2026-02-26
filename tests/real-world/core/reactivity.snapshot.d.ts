import { IfAny } from "@vue/shared";

//#region tests/real-world/core/reactivity/constants.d.ts
declare enum TrackOpTypes {
  GET = "get",
  HAS = "has",
  ITERATE = "iterate"
}
declare enum TriggerOpTypes {
  SET = "set",
  ADD = "add",
  DELETE = "delete",
  CLEAR = "clear"
}
declare enum ReactiveFlags {
  SKIP = "__v_skip",
  IS_REACTIVE = "__v_isReactive",
  IS_READONLY = "__v_isReadonly",
  IS_SHALLOW = "__v_isShallow",
  RAW = "__v_raw",
  IS_REF = "__v_isRef"
}
//#endregion
//#region tests/real-world/core/reactivity/reactive.d.ts
type UnwrapNestedRefs<T> = T extends Ref ? T : UnwrapRefSimple<T>;
declare const ReactiveMarkerSymbol: unique symbol;
interface ReactiveMarker {
  [ReactiveMarkerSymbol]?: void;
}
type Reactive<T> = UnwrapNestedRefs<T> & (T extends readonly any[] ? ReactiveMarker : {});
/**
* Returns a reactive proxy of the object.
*
* The reactive conversion is "deep": it affects all nested properties. A
* reactive object also deeply unwraps any properties that are refs while
* maintaining reactivity.
*
* @example
* ```js
* const obj = reactive({ count: 0 })
* ```
*
* @param target - The source object.
* @see {@link https://vuejs.org/api/reactivity-core.html#reactive}
*/
declare function reactive<T extends object>(target: T): Reactive<T>;
declare const ShallowReactiveMarker: unique symbol;
type ShallowReactive<T> = T & {
  [ShallowReactiveMarker]?: true;
};
/**
* Shallow version of {@link reactive}.
*
* Unlike {@link reactive}, there is no deep conversion: only root-level
* properties are reactive for a shallow reactive object. Property values are
* stored and exposed as-is - this also means properties with ref values will
* not be automatically unwrapped.
*
* @example
* ```js
* const state = shallowReactive({
*   foo: 1,
*   nested: {
*     bar: 2
*   }
* })
*
* // mutating state's own properties is reactive
* state.foo++
*
* // ...but does not convert nested objects
* isReactive(state.nested) // false
*
* // NOT reactive
* state.nested.bar++
* ```
*
* @param target - The source object.
* @see {@link https://vuejs.org/api/reactivity-advanced.html#shallowreactive}
*/
declare function shallowReactive<T extends object>(target: T): ShallowReactive<T>;
type Primitive = string | number | boolean | bigint | symbol | undefined | null;
type Builtin = Primitive | Function | Date | Error | RegExp;
type DeepReadonly<T> = T extends Builtin ? T : T extends Map<infer K, infer V> ? ReadonlyMap<DeepReadonly<K>, DeepReadonly<V>> : T extends ReadonlyMap<infer K, infer V> ? ReadonlyMap<DeepReadonly<K>, DeepReadonly<V>> : T extends WeakMap<infer K, infer V> ? WeakMap<DeepReadonly<K>, DeepReadonly<V>> : T extends Set<infer U> ? ReadonlySet<DeepReadonly<U>> : T extends ReadonlySet<infer U> ? ReadonlySet<DeepReadonly<U>> : T extends WeakSet<infer U> ? WeakSet<DeepReadonly<U>> : T extends Promise<infer U> ? Promise<DeepReadonly<U>> : T extends Ref<infer U, unknown> ? Readonly<Ref<DeepReadonly<U>>> : T extends {} ? { readonly [K in keyof T] : DeepReadonly<T[K]> } : Readonly<T>;
/**
* Takes an object (reactive or plain) or a ref and returns a readonly proxy to
* the original.
*
* A readonly proxy is deep: any nested property accessed will be readonly as
* well. It also has the same ref-unwrapping behavior as {@link reactive},
* except the unwrapped values will also be made readonly.
*
* @example
* ```js
* const original = reactive({ count: 0 })
*
* const copy = readonly(original)
*
* watchEffect(() => {
*   // works for reactivity tracking
*   console.log(copy.count)
* })
*
* // mutating original will trigger watchers relying on the copy
* original.count++
*
* // mutating the copy will fail and result in a warning
* copy.count++ // warning!
* ```
*
* @param target - The source object.
* @see {@link https://vuejs.org/api/reactivity-core.html#readonly}
*/
declare function readonly<T extends object>(target: T): DeepReadonly<UnwrapNestedRefs<T>>;
/**
* Shallow version of {@link readonly}.
*
* Unlike {@link readonly}, there is no deep conversion: only root-level
* properties are made readonly. Property values are stored and exposed as-is -
* this also means properties with ref values will not be automatically
* unwrapped.
*
* @example
* ```js
* const state = shallowReadonly({
*   foo: 1,
*   nested: {
*     bar: 2
*   }
* })
*
* // mutating state's own properties will fail
* state.foo++
*
* // ...but works on nested objects
* isReadonly(state.nested) // false
*
* // works
* state.nested.bar++
* ```
*
* @param target - The source object.
* @see {@link https://vuejs.org/api/reactivity-advanced.html#shallowreadonly}
*/
declare function shallowReadonly<T extends object>(target: T): Readonly<T>;
/**
* Checks if an object is a proxy created by {@link reactive} or
* {@link shallowReactive} (or {@link ref} in some cases).
*
* @example
* ```js
* isReactive(reactive({}))            // => true
* isReactive(readonly(reactive({})))  // => true
* isReactive(ref({}).value)           // => true
* isReactive(readonly(ref({})).value) // => true
* isReactive(ref(true))               // => false
* isReactive(shallowRef({}).value)    // => false
* isReactive(shallowReactive({}))     // => true
* ```
*
* @param value - The value to check.
* @see {@link https://vuejs.org/api/reactivity-utilities.html#isreactive}
*/
declare function isReactive(value: unknown): boolean;
/**
* Checks whether the passed value is a readonly object. The properties of a
* readonly object can change, but they can't be assigned directly via the
* passed object.
*
* The proxies created by {@link readonly} and {@link shallowReadonly} are
* both considered readonly, as is a computed ref without a set function.
*
* @param value - The value to check.
* @see {@link https://vuejs.org/api/reactivity-utilities.html#isreadonly}
*/
declare function isReadonly(value: unknown): boolean;
declare function isShallow(value: unknown): boolean;
/**
* Checks if an object is a proxy created by {@link reactive},
* {@link readonly}, {@link shallowReactive} or {@link shallowReadonly}.
*
* @param value - The value to check.
* @see {@link https://vuejs.org/api/reactivity-utilities.html#isproxy}
*/
declare function isProxy(value: any): boolean;
/**
* Returns the raw, original object of a Vue-created proxy.
*
* `toRaw()` can return the original object from proxies created by
* {@link reactive}, {@link readonly}, {@link shallowReactive} or
* {@link shallowReadonly}.
*
* This is an escape hatch that can be used to temporarily read without
* incurring proxy access / tracking overhead or write without triggering
* changes. It is **not** recommended to hold a persistent reference to the
* original object. Use with caution.
*
* @example
* ```js
* const foo = {}
* const reactiveFoo = reactive(foo)
*
* console.log(toRaw(reactiveFoo) === foo) // true
* ```
*
* @param observed - The object for which the "raw" value is requested.
* @see {@link https://vuejs.org/api/reactivity-advanced.html#toraw}
*/
declare function toRaw<T>(observed: T): T;
type Raw<T> = T & {
  [RawSymbol]?: true;
};
/**
* Marks an object so that it will never be converted to a proxy. Returns the
* object itself.
*
* @example
* ```js
* const foo = markRaw({})
* console.log(isReactive(reactive(foo))) // false
*
* // also works when nested inside other reactive objects
* const bar = reactive({ foo })
* console.log(isReactive(bar.foo)) // false
* ```
*
* **Warning:** `markRaw()` together with the shallow APIs such as
* {@link shallowReactive} allow you to selectively opt-out of the default
* deep reactive/readonly conversion and embed raw, non-proxied objects in your
* state graph.
*
* @param value - The object to be marked as "raw".
* @see {@link https://vuejs.org/api/reactivity-advanced.html#markraw}
*/
declare function markRaw<T extends object>(value: T): Raw<T>;
/**
* Returns a reactive proxy of the given value (if possible).
*
* If the given value is not an object, the original value itself is returned.
*
* @param value - The value for which a reactive proxy shall be created.
*/
declare const toReactive: <T extends unknown>(value: T) => T;
/**
* Returns a readonly proxy of the given value (if possible).
*
* If the given value is not an object, the original value itself is returned.
*
* @param value - The value for which a readonly proxy shall be created.
*/
declare const toReadonly: <T extends unknown>(value: T) => DeepReadonly<T>;
//#endregion
//#region tests/real-world/core/reactivity/effect.d.ts
type EffectScheduler = (...args: any[]) => any;
type DebuggerEvent = {
  effect: Subscriber;
} & DebuggerEventExtraInfo;
type DebuggerEventExtraInfo = {
  target: object;
  type: TrackOpTypes | TriggerOpTypes;
  key: any;
  newValue?: any;
  oldValue?: any;
  oldTarget?: Map<any, any> | Set<any>;
};
interface DebuggerOptions {
  onTrack?: (event: DebuggerEvent) => void;
  onTrigger?: (event: DebuggerEvent) => void;
}
interface ReactiveEffectOptions extends DebuggerOptions {
  scheduler?: EffectScheduler;
  allowRecurse?: boolean;
  onStop?: () => void;
}
interface ReactiveEffectRunner<T = any> {
  (): T;
  effect: ReactiveEffect;
}
declare enum EffectFlags {
  /**
  * ReactiveEffect only
  */
  ACTIVE = 1,
  RUNNING = 2,
  TRACKING = 4,
  NOTIFIED = 8,
  DIRTY = 16,
  ALLOW_RECURSE = 32,
  PAUSED = 64,
  EVALUATED = 128
}
/**
* Subscriber is a type that tracks (or subscribes to) a list of deps.
*/
interface Subscriber extends DebuggerOptions {}
declare class ReactiveEffect<T = any> implements Subscriber, ReactiveEffectOptions {
  fn: () => T;
  scheduler?: EffectScheduler;
  onStop?: () => void;
  onTrack?: (event: DebuggerEvent) => void;
  onTrigger?: (event: DebuggerEvent) => void;
  constructor(fn: () => T);
  pause(): void;
  resume(): void;
  run(): T;
  stop(): void;
  trigger(): void;
  get dirty(): boolean;
}
declare function effect<T = any>(fn: () => T, options?: ReactiveEffectOptions): ReactiveEffectRunner<T>;
/**
* Stops the effect associated with the given runner.
*
* @param runner - Association with the effect to stop tracking.
*/
declare function stop(runner: ReactiveEffectRunner): void;
/**
* Temporarily pauses tracking.
*/
declare function pauseTracking(): void;
/**
* Re-enables effect tracking (if it was paused).
*/
declare function enableTracking(): void;
/**
* Resets the previous global effect tracking state.
*/
declare function resetTracking(): void;
/**
* Registers a cleanup function for the current active effect.
* The cleanup function is called right before the next effect run, or when the
* effect is stopped.
*
* Throws a warning if there is no current active effect. The warning can be
* suppressed by passing `true` to the second argument.
*
* @param fn - the cleanup function to be registered
* @param failSilently - if `true`, will not throw warning when called without
* an active effect.
*/
declare function onEffectCleanup(fn: () => void, failSilently?: boolean): void;
//#endregion
//#region tests/real-world/core/reactivity/computed.d.ts
declare const ComputedRefSymbol: unique symbol;
declare const WritableComputedRefSymbol: unique symbol;
interface BaseComputedRef<
  T,
  S = T
> extends Ref<T, S> {
  [ComputedRefSymbol]: true;
  /**
  * @deprecated computed no longer uses effect
  */
  effect: ComputedRefImpl;
}
interface ComputedRef<T = any> extends BaseComputedRef<T> {
  readonly value: T;
}
interface WritableComputedRef<
  T,
  S = T
> extends BaseComputedRef<T, S> {
  [WritableComputedRefSymbol]: true;
}
type ComputedGetter<T> = (oldValue?: T) => T;
type ComputedSetter<T> = (newValue: T) => void;
interface WritableComputedOptions<
  T,
  S = T
> {
  get: ComputedGetter<T>;
  set: ComputedSetter<S>;
}
/**
* @private exported by @vue/reactivity for Vue core use, but not exported from
* the main vue package
*/
declare class ComputedRefImpl<T = any> implements Subscriber {
  fn: ComputedGetter<T>;
  private readonly setter;
  effect: this;
  onTrack?: (event: DebuggerEvent) => void;
  onTrigger?: (event: DebuggerEvent) => void;
  constructor(fn: ComputedGetter<T>, setter: ComputedSetter<T> | undefined, isSSR: boolean);
  get value(): T;
  set value(newValue: T);
}
/**
* Takes a getter function and returns a readonly reactive ref object for the
* returned value from the getter. It can also take an object with get and set
* functions to create a writable ref object.
*
* @example
* ```js
* // Creating a readonly computed ref:
* const count = ref(1)
* const plusOne = computed(() => count.value + 1)
*
* console.log(plusOne.value) // 2
* plusOne.value++ // error
* ```
*
* ```js
* // Creating a writable computed ref:
* const count = ref(1)
* const plusOne = computed({
*   get: () => count.value + 1,
*   set: (val) => {
*     count.value = val - 1
*   }
* })
*
* plusOne.value = 1
* console.log(count.value) // 0
* ```
*
* @param getter - Function that produces the next value.
* @param debugOptions - For debugging. See {@link https://vuejs.org/guide/extras/reactivity-in-depth.html#computed-debugging}.
* @see {@link https://vuejs.org/api/reactivity-core.html#computed}
*/
declare function computed<T>(getter: ComputedGetter<T>, debugOptions?: DebuggerOptions): ComputedRef<T>;
declare function computed<
  T,
  S = T
>(options: WritableComputedOptions<T, S>, debugOptions?: DebuggerOptions): WritableComputedRef<T, S>;
//#endregion
//#region tests/real-world/core/reactivity/ref.d.ts
declare const RefSymbol: unique symbol;
declare const RawSymbol: unique symbol;
interface Ref<
  T = any,
  S = T
> {
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
declare function isRef<T>(r: Ref<T> | unknown): r is Ref<T>;
/**
* Takes an inner value and returns a reactive and mutable ref object, which
* has a single property `.value` that points to the inner value.
*
* @param value - The object to wrap in the ref.
* @see {@link https://vuejs.org/api/reactivity-core.html#ref}
*/
declare function ref<T>(value: T): [T] extends [Ref] ? IfAny<T, Ref<T>, T> : Ref<UnwrapRef<T>, UnwrapRef<T> | T>;
declare function ref<T = any>(): Ref<T | undefined>;
declare const ShallowRefMarker: unique symbol;
type ShallowRef<
  T = any,
  S = T
> = Ref<T, S> & {
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
declare function shallowRef<T>(value: T): Ref extends T ? T extends Ref ? IfAny<T, ShallowRef<T>, T> : ShallowRef<T> : ShallowRef<T>;
declare function shallowRef<T = any>(): ShallowRef<T | undefined>;
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
declare function triggerRef(ref: Ref): void;
type MaybeRef<T = any> = T | Ref<T> | ShallowRef<T> | WritableComputedRef<T>;
type MaybeRefOrGetter<T = any> = MaybeRef<T> | ComputedRef<T> | (() => T);
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
declare function unref<T>(ref: MaybeRef<T> | ComputedRef<T>): T;
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
declare function toValue<T>(source: MaybeRefOrGetter<T>): T;
/**
* Returns a proxy for the given object that shallowly unwraps properties that
* are refs. If the object already is reactive, it's returned as-is. If not, a
* new reactive proxy is created.
*
* @param objectWithRefs - Either an already-reactive object or a simple object
* that contains refs.
*/
declare function proxyRefs<T extends object>(objectWithRefs: T): ShallowUnwrapRef<T>;
type CustomRefFactory<T> = (track: () => void, trigger: () => void) => {
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
declare function customRef<T>(factory: CustomRefFactory<T>): Ref<T>;
type ToRefs<T = any> = { [K in keyof T] : ToRef<T[K]> };
/**
* Converts a reactive object to a plain object where each property of the
* resulting object is a ref pointing to the corresponding property of the
* original object. Each individual ref is created using {@link toRef}.
*
* @param object - Reactive object to be made into an object of linked refs.
* @see {@link https://vuejs.org/api/reactivity-utilities.html#torefs}
*/
declare function toRefs<T extends object>(object: T): ToRefs<T>;
type ToRef<T> = IfAny<T, Ref<T>, [T] extends [Ref] ? T : Ref<T>>;
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
declare function toRef<T>(value: T): T extends () => infer R ? Readonly<Ref<R>> : T extends Ref ? T : Ref<UnwrapRef<T>>;
declare function toRef<
  T extends object,
  K extends keyof T
>(object: T, key: K): ToRef<T[K]>;
declare function toRef<
  T extends object,
  K extends keyof T
>(object: T, key: K, defaultValue: T[K]): ToRef<Exclude<T[K], undefined>>;
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
interface RefUnwrapBailTypes {}
type ShallowUnwrapRef<T> = { [K in keyof T] : DistributeRef<T[K]> };
type DistributeRef<T> = T extends Ref<infer V, unknown> ? V : T;
type UnwrapRef<T> = T extends ShallowRef<infer V, unknown> ? V : T extends Ref<infer V, unknown> ? UnwrapRefSimple<V> : UnwrapRefSimple<T>;
type UnwrapRefSimple<T> = T extends Builtin | Ref | RefUnwrapBailTypes[keyof RefUnwrapBailTypes] | {
  [RawSymbol]?: true;
} ? T : T extends Map<infer K, infer V> ? Map<K, UnwrapRefSimple<V>> & UnwrapRef<Omit<T, keyof Map<any, any>>> : T extends WeakMap<infer K, infer V> ? WeakMap<K, UnwrapRefSimple<V>> & UnwrapRef<Omit<T, keyof WeakMap<any, any>>> : T extends Set<infer V> ? Set<UnwrapRefSimple<V>> & UnwrapRef<Omit<T, keyof Set<any>>> : T extends WeakSet<infer V> ? WeakSet<UnwrapRefSimple<V>> & UnwrapRef<Omit<T, keyof WeakSet<any>>> : T extends ReadonlyArray<any> ? { [K in keyof T] : UnwrapRefSimple<T[K]> } : T extends object & {
  [ShallowReactiveMarker]?: never;
} ? { [P in keyof T] : P extends symbol ? T[P] : UnwrapRef<T[P]> } : T;
//#endregion
//#region tests/real-world/core/reactivity/dep.d.ts
declare const ITERATE_KEY: unique symbol;
declare const MAP_KEY_ITERATE_KEY: unique symbol;
declare const ARRAY_ITERATE_KEY: unique symbol;
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
declare function track(target: object, type: TrackOpTypes, key: unknown): void;
/**
* Finds all deps associated with the target (or a specific property) and
* triggers the effects stored within.
*
* @param target - The reactive object.
* @param type - Defines the type of the operation that needs to trigger effects.
* @param key - Can be used to target a specific reactive property in the target object.
*/
declare function trigger(target: object, type: TriggerOpTypes, key?: unknown, newValue?: unknown, oldValue?: unknown, oldTarget?: Map<unknown, unknown> | Set<unknown>): void;
//#endregion
//#region tests/real-world/core/reactivity/effectScope.d.ts
declare class EffectScope {
  detached: boolean;
  private _isPaused;
  readonly __v_skip = true;
  constructor(detached?: boolean);
  get active(): boolean;
  pause(): void;
  /**
  * Resumes the effect scope, including all child scopes and effects.
  */
  resume(): void;
  run<T>(fn: () => T): T | undefined;
  prevScope: EffectScope | undefined;
  stop(fromParent?: boolean): void;
}
/**
* Creates an effect scope object which can capture the reactive effects (i.e.
* computed and watchers) created within it so that these effects can be
* disposed together. For detailed use cases of this API, please consult its
* corresponding {@link https://github.com/vuejs/rfcs/blob/master/active-rfcs/0041-reactivity-effect-scope.md | RFC}.
*
* @param detached - Can be used to create a "detached" effect scope.
* @see {@link https://vuejs.org/api/reactivity-advanced.html#effectscope}
*/
declare function effectScope(detached?: boolean): EffectScope;
/**
* Returns the current active effect scope if there is one.
*
* @see {@link https://vuejs.org/api/reactivity-advanced.html#getcurrentscope}
*/
declare function getCurrentScope(): EffectScope | undefined;
/**
* Registers a dispose callback on the current active effect scope. The
* callback will be invoked when the associated effect scope is stopped.
*
* @param fn - The callback function to attach to the scope's cleanup.
* @see {@link https://vuejs.org/api/reactivity-advanced.html#onscopedispose}
*/
declare function onScopeDispose(fn: () => void, failSilently?: boolean): void;
//#endregion
//#region tests/real-world/core/reactivity/arrayInstrumentations.d.ts
/**
* Track array iteration and return:
* - if input is reactive: a cloned raw array with reactive values
* - if input is non-reactive or shallowReactive: the original raw array
*/
declare function reactiveReadArray<T>(array: T[]): T[];
/**
* Track array iteration and return raw array
*/
declare function shallowReadArray<T>(arr: T[]): T[];
//#endregion
//#region tests/real-world/core/reactivity/watch.d.ts
declare enum WatchErrorCodes {
  WATCH_GETTER = 2,
  WATCH_CALLBACK = 3,
  WATCH_CLEANUP = 4
}
type WatchEffect = (onCleanup: OnCleanup) => void;
type WatchSource<T = any> = Ref<T, any> | ComputedRef<T> | (() => T);
type WatchCallback<
  V = any,
  OV = any
> = (value: V, oldValue: OV, onCleanup: OnCleanup) => any;
type OnCleanup = (cleanupFn: () => void) => void;
interface WatchOptions<Immediate = boolean> extends DebuggerOptions {
  immediate?: Immediate;
  deep?: boolean | number;
  once?: boolean;
  scheduler?: WatchScheduler;
  onWarn?: (msg: string, ...args: any[]) => void;
}
type WatchStopHandle = () => void;
interface WatchHandle extends WatchStopHandle {
  pause: () => void;
  resume: () => void;
  stop: () => void;
}
type WatchScheduler = (job: () => void, isFirstRun: boolean) => void;
/**
* Returns the current active effect if there is one.
*/
declare function getCurrentWatcher(): ReactiveEffect<any> | undefined;
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
declare function onWatcherCleanup(cleanupFn: () => void, failSilently?: boolean, owner?: ReactiveEffect | undefined): void;
declare function watch(source: WatchSource | WatchSource[] | WatchEffect | object, cb?: WatchCallback | null, options?: WatchOptions): WatchHandle;
declare function traverse(value: unknown, depth?: number, seen?: Map<unknown, number>): unknown;
//#endregion
export { ARRAY_ITERATE_KEY, type ComputedGetter, type ComputedRef, type ComputedRefImpl, type ComputedSetter, type CustomRefFactory, type DebuggerEvent, type DebuggerEventExtraInfo, type DebuggerOptions, type DeepReadonly, EffectFlags, type EffectScheduler, EffectScope, ITERATE_KEY, MAP_KEY_ITERATE_KEY, type MaybeRef, type MaybeRefOrGetter, type OnCleanup, type Raw, type Reactive, ReactiveEffect, type ReactiveEffectOptions, type ReactiveEffectRunner, ReactiveFlags, type ReactiveMarker, type Ref, type RefUnwrapBailTypes, type ShallowReactive, type ShallowRef, type ShallowUnwrapRef, type ToRef, type ToRefs, TrackOpTypes, TriggerOpTypes, type UnwrapNestedRefs, type UnwrapRef, type WatchCallback, type WatchEffect, WatchErrorCodes, type WatchHandle, type WatchOptions, type WatchScheduler, type WatchSource, type WatchStopHandle, type WritableComputedOptions, type WritableComputedRef, computed, customRef, effect, effectScope, enableTracking, getCurrentScope, getCurrentWatcher, isProxy, isReactive, isReadonly, isRef, isShallow, markRaw, onEffectCleanup, onScopeDispose, onWatcherCleanup, pauseTracking, proxyRefs, reactive, reactiveReadArray, readonly, ref, resetTracking, shallowReactive, shallowReadArray, shallowReadonly, shallowRef, stop, toRaw, toReactive, toReadonly, toRef, toRefs, toValue, track, traverse, trigger, triggerRef, unref, watch };