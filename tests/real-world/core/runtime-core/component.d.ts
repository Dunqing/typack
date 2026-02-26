import { type VNode } from './vnode';
import { type ReactiveEffect } from '@vue/reactivity';
import { type ComponentPublicInstance, type ComponentPublicInstanceConstructor } from './componentPublicInstance';
import { type ComponentPropsOptions } from './componentProps';
import { type InternalSlots, type Slots, type SlotsType, type UnwrapSlotsType } from './componentSlots';
import { type AppConfig, type AppContext } from './apiCreateApp';
import { type ComponentOptions, type ComputedOptions, type MethodOptions } from './componentOptions';
import { type EmitFn, type EmitsOptions, type EmitsToProps, type ShortEmitsToObject } from './componentEmits';
import { type IfAny } from '@vue/shared';
import type { SuspenseBoundary } from './components/Suspense';
import { type CompatConfig } from './compat/compatConfig';
import type { SchedulerJob } from './scheduler';
import type { TeleportProps } from './components/Teleport';
import type { SuspenseProps } from './components/Suspense';
import type { KeepAliveProps } from './components/KeepAlive';
import type { BaseTransitionProps } from './components/BaseTransition';
import type { DefineComponent } from './apiDefineComponent';
export type Data = Record<string, unknown>;
/**
 * Public utility type for extracting the instance type of a component.
 * Works with all valid component definition types. This is intended to replace
 * the usage of `InstanceType<typeof Comp>` which only works for
 * constructor-based component definition types.
 *
 * @example
 * ```ts
 * const MyComp = { ... }
 * declare const instance: ComponentInstance<typeof MyComp>
 * ```
 */
export type ComponentInstance<T> = T extends {
    new (): ComponentPublicInstance;
} ? InstanceType<T> : T extends FunctionalComponent<infer Props, infer Emits> ? ComponentPublicInstance<Props, {}, {}, {}, {}, ShortEmitsToObject<Emits>> : T extends Component<infer PropsOrInstance, infer RawBindings, infer D, infer C, infer M> ? PropsOrInstance extends {
    $props: unknown;
} ? PropsOrInstance : ComponentPublicInstance<unknown extends PropsOrInstance ? {} : PropsOrInstance, unknown extends RawBindings ? {} : RawBindings, unknown extends D ? {} : D, C, M> : never;
/**
 * For extending allowed non-declared props on components in TSX
 */
export interface ComponentCustomProps {
}
/**
 * For globally defined Directives
 * Here is an example of adding a directive `VTooltip` as global directive:
 *
 * @example
 * ```ts
 * import VTooltip from 'v-tooltip'
 *
 * declare module '@vue/runtime-core' {
 *   interface GlobalDirectives {
 *     VTooltip
 *   }
 * }
 * ```
 */
export interface GlobalDirectives {
}
/**
 * For globally defined Components
 * Here is an example of adding a component `RouterView` as global component:
 *
 * @example
 * ```ts
 * import { RouterView } from 'vue-router'
 *
 * declare module '@vue/runtime-core' {
 *   interface GlobalComponents {
 *     RouterView
 *   }
 * }
 * ```
 */
export interface GlobalComponents {
    Teleport: DefineComponent<TeleportProps>;
    Suspense: DefineComponent<SuspenseProps>;
    KeepAlive: DefineComponent<KeepAliveProps>;
    BaseTransition: DefineComponent<BaseTransitionProps>;
}
/**
 * Default allowed non-declared props on component in TSX
 */
export interface AllowedComponentProps {
    class?: unknown;
    style?: unknown;
}
export interface ComponentInternalOptions {
    /**
     * Compat build only, for bailing out of certain compatibility behavior
     */
    __isBuiltIn?: boolean;
    /**
     * This one should be exposed so that devtools can make use of it
     */
    __file?: string;
    /**
     * name inferred from filename
     */
    __name?: string;
}
export interface FunctionalComponent<P = {}, E extends EmitsOptions | Record<string, any[]> = {}, S extends Record<string, any> = any, EE extends EmitsOptions = ShortEmitsToObject<E>> extends ComponentInternalOptions {
    (props: P & EmitsToProps<EE>, ctx: Omit<SetupContext<EE, IfAny<S, {}, SlotsType<S>>>, 'expose'>): any;
    props?: ComponentPropsOptions<P>;
    emits?: EE | (keyof EE)[];
    slots?: IfAny<S, Slots, SlotsType<S>>;
    inheritAttrs?: boolean;
    displayName?: string;
    compatConfig?: CompatConfig;
}
export interface ClassComponent {
    new (...args: any[]): ComponentPublicInstance<any, any, any, any, any>;
    __vccOpts: ComponentOptions;
}
/**
 * Concrete component type matches its actual value: it's either an options
 * object, or a function. Use this where the code expects to work with actual
 * values, e.g. checking if its a function or not. This is mostly for internal
 * implementation code.
 */
export type ConcreteComponent<Props = {}, RawBindings = any, D = any, C extends ComputedOptions = ComputedOptions, M extends MethodOptions = MethodOptions, E extends EmitsOptions | Record<string, any[]> = {}, S extends Record<string, any> = any> = ComponentOptions<Props, RawBindings, D, C, M> | FunctionalComponent<Props, E, S>;
/**
 * A type used in public APIs where a component type is expected.
 * The constructor type is an artificial type returned by defineComponent().
 */
export type Component<PropsOrInstance = any, RawBindings = any, D = any, C extends ComputedOptions = ComputedOptions, M extends MethodOptions = MethodOptions, E extends EmitsOptions | Record<string, any[]> = {}, S extends Record<string, any> = any> = ConcreteComponent<PropsOrInstance, RawBindings, D, C, M, E, S> | ComponentPublicInstanceConstructor<PropsOrInstance>;
export type { ComponentOptions };
export type LifecycleHook<TFn = Function> = (TFn & SchedulerJob)[] | null;
export type SetupContext<E = EmitsOptions, S extends SlotsType = {}> = E extends any ? {
    attrs: Data;
    slots: UnwrapSlotsType<S>;
    emit: EmitFn<E>;
    expose: <Exposed extends Record<string, any> = Record<string, any>>(exposed?: Exposed) => void;
} : never;
/**
 * We expose a subset of properties on the internal instance as they are
 * useful for advanced external libraries and tools.
 */
export interface ComponentInternalInstance {
    uid: number;
    type: ConcreteComponent;
    parent: ComponentInternalInstance | null;
    root: ComponentInternalInstance;
    appContext: AppContext;
    /**
     * Vnode representing this component in its parent's vdom tree
     */
    vnode: VNode;
    /**
     * Root vnode of this component's own vdom tree
     */
    subTree: VNode;
    /**
     * Render effect instance
     */
    effect: ReactiveEffect;
    /**
     * Force update render effect
     */
    update: () => void;
    /**
     * Render effect job to be passed to scheduler (checks if dirty)
     */
    job: SchedulerJob;
    proxy: ComponentPublicInstance | null;
    exposed: Record<string, any> | null;
    exposeProxy: Record<string, any> | null;
    data: Data;
    props: Data;
    attrs: Data;
    slots: InternalSlots;
    refs: Data;
    emit: EmitFn;
    isMounted: boolean;
    isUnmounted: boolean;
    isDeactivated: boolean;
}
export declare function createComponentInstance(vnode: VNode, parent: ComponentInternalInstance | null, suspense: SuspenseBoundary | null): ComponentInternalInstance;
export declare let currentInstance: ComponentInternalInstance | null;
export declare const getCurrentInstance: () => ComponentInternalInstance | null;
export declare const setCurrentInstance: (instance: ComponentInternalInstance) => () => void;
export declare const unsetCurrentInstance: () => void;
export declare function validateComponentName(name: string, { isNativeTag }: AppConfig): void;
export declare function isStatefulComponent(instance: ComponentInternalInstance): number;
export declare let isInSSRComponentSetup: boolean;
export declare function setupComponent(instance: ComponentInternalInstance, isSSR?: boolean, optimized?: boolean): Promise<void> | undefined;
export declare function handleSetupResult(instance: ComponentInternalInstance, setupResult: unknown, isSSR: boolean): void;
/**
 * For runtime-dom to register the compiler.
 * Note the exported method uses any to avoid d.ts relying on the compiler types.
 */
export declare function registerRuntimeCompiler(_compile: any): void;
export declare const isRuntimeOnly: () => boolean;
export declare function finishComponentSetup(instance: ComponentInternalInstance, isSSR: boolean, skipOptions?: boolean): void;
export declare function createSetupContext(instance: ComponentInternalInstance): SetupContext;
export declare function getComponentPublicInstance(instance: ComponentInternalInstance): ComponentPublicInstance | ComponentInternalInstance['exposed'] | null;
export declare function getComponentName(Component: ConcreteComponent, includeInferred?: boolean): string | false | undefined;
export declare function formatComponentName(instance: ComponentInternalInstance | null, Component: ConcreteComponent, isRoot?: boolean): string;
export declare function isClassComponent(value: unknown): value is ClassComponent;
export interface ComponentCustomElementInterface {
}
