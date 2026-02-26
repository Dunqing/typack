import { type Component, type ComponentInternalInstance, type ComponentInternalOptions, type Data, type SetupContext } from './component';
import { type LooseRequired, type Prettify } from '@vue/shared';
import { type WatchCallback, type WatchOptions } from './apiWatch';
import { type DebuggerHook, type ErrorCapturedHook } from './apiLifecycle';
import { type ComputedGetter, type WritableComputedOptions } from '@vue/reactivity';
import type { ComponentObjectPropsOptions, ExtractDefaultPropTypes, ExtractPropTypes } from './componentProps';
import type { EmitsOptions, EmitsToProps, TypeEmitsToOptions } from './componentEmits';
import type { Directive } from './directives';
import { type ComponentPublicInstance, type CreateComponentPublicInstanceWithMixins, type IntersectionMixin, type UnwrapMixinsType } from './componentPublicInstance';
import type { VNodeChild } from './vnode';
import { type CompatConfig } from './compat/compatConfig';
import type { OptionMergeFunction } from './apiCreateApp';
import type { SlotsType } from './componentSlots';
import { type ComponentTypeEmits } from './apiSetupHelpers';
/**
 * Interface for declaring custom options.
 *
 * @example
 * ```ts
 * declare module 'vue' {
 *   interface ComponentCustomOptions {
 *     beforeRouteUpdate?(
 *       to: Route,
 *       from: Route,
 *       next: () => void
 *     ): void
 *   }
 * }
 * ```
 */
export interface ComponentCustomOptions {
}
export type RenderFunction = () => VNodeChild;
export interface ComponentOptionsBase<Props, RawBindings, D, C extends ComputedOptions, M extends MethodOptions, Mixin extends ComponentOptionsMixin, Extends extends ComponentOptionsMixin, E extends EmitsOptions, EE extends string = string, Defaults = {}, I extends ComponentInjectOptions = {}, II extends string = string, S extends SlotsType = {}, LC extends Record<string, Component> = {}, Directives extends Record<string, Directive> = {}, Exposed extends string = string, Provide extends ComponentProvideOptions = ComponentProvideOptions> extends LegacyOptions<Props, D, C, M, Mixin, Extends, I, II, Provide>, ComponentInternalOptions, ComponentCustomOptions {
    setup?: (this: void, props: LooseRequired<Props & Prettify<UnwrapMixinsType<IntersectionMixin<Mixin> & IntersectionMixin<Extends>, 'P'>>>, ctx: SetupContext<E, S>) => Promise<RawBindings> | RawBindings | RenderFunction | void;
    name?: string;
    template?: string | object;
    render?: Function;
    components?: LC & Record<string, Component>;
    directives?: Directives & Record<string, Directive>;
    inheritAttrs?: boolean;
    emits?: (E | EE[]) & ThisType<void>;
    slots?: S;
    expose?: Exposed[];
    serverPrefetch?(): void | Promise<any>;
    compilerOptions?: RuntimeCompilerOptions;
    call?: (this: unknown, ...args: unknown[]) => never;
    __isFragment?: never;
    __isTeleport?: never;
    __isSuspense?: never;
    __defaults?: Defaults;
}
/**
 * Subset of compiler options that makes sense for the runtime.
 */
export interface RuntimeCompilerOptions {
    isCustomElement?: (tag: string) => boolean;
    whitespace?: 'preserve' | 'condense';
    comments?: boolean;
    delimiters?: [string, string];
}
export type ComponentOptions<Props = {}, RawBindings = any, D = any, C extends ComputedOptions = any, M extends MethodOptions = any, Mixin extends ComponentOptionsMixin = any, Extends extends ComponentOptionsMixin = any, E extends EmitsOptions = any, EE extends string = string, Defaults = {}, I extends ComponentInjectOptions = {}, II extends string = string, S extends SlotsType = {}, LC extends Record<string, Component> = {}, Directives extends Record<string, Directive> = {}, Exposed extends string = string, Provide extends ComponentProvideOptions = ComponentProvideOptions> = ComponentOptionsBase<Props, RawBindings, D, C, M, Mixin, Extends, E, EE, Defaults, I, II, S, LC, Directives, Exposed, Provide> & ThisType<CreateComponentPublicInstanceWithMixins<{}, RawBindings, D, C, M, Mixin, Extends, E, Readonly<Props>, Defaults, false, I, S, LC, Directives>>;
export type ComponentOptionsMixin = ComponentOptionsBase<any, any, any, any, any, any, any, any, any, any, any, any, any, any, any, any, any>;
export type ComputedOptions = Record<string, ComputedGetter<any> | WritableComputedOptions<any>>;
export interface MethodOptions {
    [key: string]: Function;
}
export type ExtractComputedReturns<T extends any> = {
    [key in keyof T]: T[key] extends {
        get: (...args: any[]) => infer TReturn;
    } ? TReturn : T[key] extends (...args: any[]) => infer TReturn ? TReturn : never;
};
export type ObjectWatchOptionItem = {
    handler: WatchCallback | string;
} & WatchOptions;
type WatchOptionItem = string | WatchCallback | ObjectWatchOptionItem;
type ComponentWatchOptionItem = WatchOptionItem | WatchOptionItem[];
type ComponentWatchOptions = Record<string, ComponentWatchOptionItem>;
export type ComponentProvideOptions = ObjectProvideOptions | Function;
type ObjectProvideOptions = Record<string | symbol, unknown>;
export type ComponentInjectOptions = string[] | ObjectInjectOptions;
type ObjectInjectOptions = Record<string | symbol, string | symbol | {
    from?: string | symbol;
    default?: unknown;
}>;
export type InjectToObject<T extends ComponentInjectOptions> = T extends string[] ? {
    [K in T[number]]?: unknown;
} : T extends ObjectInjectOptions ? {
    [K in keyof T]?: unknown;
} : never;
interface LegacyOptions<Props, D, C extends ComputedOptions, M extends MethodOptions, Mixin extends ComponentOptionsMixin, Extends extends ComponentOptionsMixin, I extends ComponentInjectOptions, II extends string, Provide extends ComponentProvideOptions = ComponentProvideOptions> {
    compatConfig?: CompatConfig;
    [key: string]: any;
    data?: (this: CreateComponentPublicInstanceWithMixins<Props, {}, {}, {}, MethodOptions, Mixin, Extends>, vm: CreateComponentPublicInstanceWithMixins<Props, {}, {}, {}, MethodOptions, Mixin, Extends>) => D;
    computed?: C;
    methods?: M;
    watch?: ComponentWatchOptions;
    provide?: Provide;
    inject?: I | II[];
    filters?: Record<string, Function>;
    mixins?: Mixin[];
    extends?: Extends;
    beforeCreate?(): any;
    created?(): any;
    beforeMount?(): any;
    mounted?(): any;
    beforeUpdate?(): any;
    updated?(): any;
    activated?(): any;
    deactivated?(): any;
    /** @deprecated use `beforeUnmount` instead */
    beforeDestroy?(): any;
    beforeUnmount?(): any;
    /** @deprecated use `unmounted` instead */
    destroyed?(): any;
    unmounted?(): any;
    renderTracked?: DebuggerHook;
    renderTriggered?: DebuggerHook;
    errorCaptured?: ErrorCapturedHook;
    /**
     * runtime compile only
     * @deprecated use `compilerOptions.delimiters` instead.
     */
    delimiters?: [string, string];
    /**
     * #3468
     *
     * type-only, used to assist Mixin's type inference,
     * TypeScript will try to simplify the inferred `Mixin` type,
     * with the `__differentiator`, TypeScript won't be able to combine different mixins,
     * because the `__differentiator` will be different
     */
    __differentiator?: keyof D | keyof C | keyof M;
}
type MergedHook<T = () => void> = T | T[];
export type MergedComponentOptions = ComponentOptions & MergedComponentOptionsOverride;
export type MergedComponentOptionsOverride = {
    beforeCreate?: MergedHook;
    created?: MergedHook;
    beforeMount?: MergedHook;
    mounted?: MergedHook;
    beforeUpdate?: MergedHook;
    updated?: MergedHook;
    activated?: MergedHook;
    deactivated?: MergedHook;
    /** @deprecated use `beforeUnmount` instead */
    beforeDestroy?: MergedHook;
    beforeUnmount?: MergedHook;
    /** @deprecated use `unmounted` instead */
    destroyed?: MergedHook;
    unmounted?: MergedHook;
    renderTracked?: MergedHook<DebuggerHook>;
    renderTriggered?: MergedHook<DebuggerHook>;
    errorCaptured?: MergedHook<ErrorCapturedHook>;
};
export type OptionTypesKeys = 'P' | 'B' | 'D' | 'C' | 'M' | 'Defaults';
export type OptionTypesType<P = {}, B = {}, D = {}, C extends ComputedOptions = {}, M extends MethodOptions = {}, Defaults = {}> = {
    P: P;
    B: B;
    D: D;
    C: C;
    M: M;
    Defaults: Defaults;
};
export declare let shouldCacheAccess: boolean;
export declare function applyOptions(instance: ComponentInternalInstance): void;
export declare function resolveInjections(injectOptions: ComponentInjectOptions, ctx: any, checkDuplicateProperties?: any): void;
export declare function createWatcher(raw: ComponentWatchOptionItem, ctx: Data, publicThis: ComponentPublicInstance, key: string): void;
/**
 * Resolve merged options and cache it on the component.
 * This is done only once per-component since the merging does not involve
 * instances.
 */
export declare function resolveMergedOptions(instance: ComponentInternalInstance): MergedComponentOptions;
export declare function mergeOptions(to: any, from: any, strats: Record<string, OptionMergeFunction>, asMixin?: boolean): any;
export declare const internalOptionMergeStrats: Record<string, Function>;
/**
 * @deprecated
 */
export type ComponentOptionsWithoutProps<Props = {}, RawBindings = {}, D = {}, C extends ComputedOptions = {}, M extends MethodOptions = {}, Mixin extends ComponentOptionsMixin = ComponentOptionsMixin, Extends extends ComponentOptionsMixin = ComponentOptionsMixin, E extends EmitsOptions = {}, EE extends string = string, I extends ComponentInjectOptions = {}, II extends string = string, S extends SlotsType = {}, LC extends Record<string, Component> = {}, Directives extends Record<string, Directive> = {}, Exposed extends string = string, Provide extends ComponentProvideOptions = ComponentProvideOptions, TE extends ComponentTypeEmits = {}, ResolvedEmits extends EmitsOptions = {} extends E ? TypeEmitsToOptions<TE> : E, PE = Props & EmitsToProps<ResolvedEmits>> = ComponentOptionsBase<PE, RawBindings, D, C, M, Mixin, Extends, E, EE, {}, I, II, S, LC, Directives, Exposed, Provide> & {
    props?: never;
    /**
     * @private for language-tools use only
     */
    __typeProps?: Props;
    /**
     * @private for language-tools use only
     */
    __typeEmits?: TE;
} & ThisType<CreateComponentPublicInstanceWithMixins<PE, RawBindings, D, C, M, Mixin, Extends, ResolvedEmits, EE, {}, false, I, S, LC, Directives, string>>;
/**
 * @deprecated
 */
export type ComponentOptionsWithArrayProps<PropNames extends string = string, RawBindings = {}, D = {}, C extends ComputedOptions = {}, M extends MethodOptions = {}, Mixin extends ComponentOptionsMixin = ComponentOptionsMixin, Extends extends ComponentOptionsMixin = ComponentOptionsMixin, E extends EmitsOptions = EmitsOptions, EE extends string = string, I extends ComponentInjectOptions = {}, II extends string = string, S extends SlotsType = {}, LC extends Record<string, Component> = {}, Directives extends Record<string, Directive> = {}, Exposed extends string = string, Provide extends ComponentProvideOptions = ComponentProvideOptions, Props = Prettify<Readonly<{
    [key in PropNames]?: any;
} & EmitsToProps<E>>>> = ComponentOptionsBase<Props, RawBindings, D, C, M, Mixin, Extends, E, EE, {}, I, II, S, LC, Directives, Exposed, Provide> & {
    props: PropNames[];
} & ThisType<CreateComponentPublicInstanceWithMixins<Props, RawBindings, D, C, M, Mixin, Extends, E, Props, {}, false, I, S, LC, Directives, string>>;
/**
 * @deprecated
 */
export type ComponentOptionsWithObjectProps<PropsOptions = ComponentObjectPropsOptions, RawBindings = {}, D = {}, C extends ComputedOptions = {}, M extends MethodOptions = {}, Mixin extends ComponentOptionsMixin = ComponentOptionsMixin, Extends extends ComponentOptionsMixin = ComponentOptionsMixin, E extends EmitsOptions = EmitsOptions, EE extends string = string, I extends ComponentInjectOptions = {}, II extends string = string, S extends SlotsType = {}, LC extends Record<string, Component> = {}, Directives extends Record<string, Directive> = {}, Exposed extends string = string, Provide extends ComponentProvideOptions = ComponentProvideOptions, Props = Prettify<Readonly<ExtractPropTypes<PropsOptions>> & Readonly<EmitsToProps<E>>>, Defaults = ExtractDefaultPropTypes<PropsOptions>> = ComponentOptionsBase<Props, RawBindings, D, C, M, Mixin, Extends, E, EE, Defaults, I, II, S, LC, Directives, Exposed, Provide> & {
    props: PropsOptions & ThisType<void>;
} & ThisType<CreateComponentPublicInstanceWithMixins<Props, RawBindings, D, C, M, Mixin, Extends, E, Props, Defaults, false, I, S, LC, Directives>>;
export {};
