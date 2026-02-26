export { ref, shallowRef, isRef, toRef, toValue, toRefs, unref, proxyRefs, customRef, triggerRef, type Ref, type MaybeRef, type MaybeRefOrGetter, type ToRef, type ToRefs, type UnwrapRef, type ShallowRef, type ShallowUnwrapRef, type RefUnwrapBailTypes, type CustomRefFactory, } from './reactivity/ref';
export { reactive, readonly, isReactive, isReadonly, isShallow, isProxy, shallowReactive, shallowReadonly, markRaw, toRaw, toReactive, toReadonly, type Raw, type DeepReadonly, type ShallowReactive, type UnwrapNestedRefs, type Reactive, type ReactiveMarker, } from './reactivity/reactive';
export { computed, type ComputedRef, type WritableComputedRef, type WritableComputedOptions, type ComputedGetter, type ComputedSetter, type ComputedRefImpl, } from './reactivity/computed';
export { effect, stop, enableTracking, pauseTracking, resetTracking, onEffectCleanup, ReactiveEffect, EffectFlags, type ReactiveEffectRunner, type ReactiveEffectOptions, type EffectScheduler, type DebuggerOptions, type DebuggerEvent, type DebuggerEventExtraInfo, } from './reactivity/effect';
export { trigger, track, ITERATE_KEY, ARRAY_ITERATE_KEY, MAP_KEY_ITERATE_KEY, } from './reactivity/dep';
export { effectScope, EffectScope, getCurrentScope, onScopeDispose, } from './reactivity/effectScope';
export { reactiveReadArray, shallowReadArray } from './reactivity/arrayInstrumentations';
export { TrackOpTypes, TriggerOpTypes, ReactiveFlags } from './reactivity/constants';
export { watch, getCurrentWatcher, traverse, onWatcherCleanup, WatchErrorCodes, type WatchOptions, type WatchScheduler, type WatchStopHandle, type WatchHandle, type WatchEffect, type WatchSource, type WatchCallback, type OnCleanup, } from './reactivity/watch';
