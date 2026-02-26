import { type Component, type ComponentInternalInstance, type ConcreteComponent, type Data } from './component';
import type { ComponentOptions, RuntimeCompilerOptions } from './componentOptions';
import type { ComponentCustomProperties, ComponentPublicInstance } from './componentPublicInstance';
import { type Directive } from './directives';
import type { ElementNamespace, RootRenderFunction } from './renderer';
import type { InjectionKey } from './apiInject';
import { type VNode } from './vnode';
import type { RootHydrateFunction } from './hydration';
import type { DefineComponent } from './apiDefineComponent';
export interface App<HostElement = any> {
    version: string;
    config: AppConfig;
    use<Options extends unknown[]>(plugin: Plugin<Options>, ...options: NoInfer<Options>): this;
    use<Options>(plugin: Plugin<Options>, options: NoInfer<Options>): this;
    mixin(mixin: ComponentOptions): this;
    component(name: string): Component | undefined;
    component<T extends Component | DefineComponent>(name: string, component: T): this;
    directive<HostElement = any, Value = any, Modifiers extends string = string, Arg = any>(name: string): Directive<HostElement, Value, Modifiers, Arg> | undefined;
    directive<HostElement = any, Value = any, Modifiers extends string = string, Arg = any>(name: string, directive: Directive<HostElement, Value, Modifiers, Arg>): this;
    mount(rootContainer: HostElement | string, 
    /**
     * @internal
     */
    isHydrate?: boolean, 
    /**
     * @internal
     */
    namespace?: boolean | ElementNamespace, 
    /**
     * @internal
     */
    vnode?: VNode): ComponentPublicInstance;
    unmount(): void;
    onUnmount(cb: () => void): void;
    provide<T, K = InjectionKey<T> | string | number>(key: K, value: K extends InjectionKey<infer V> ? V : T): this;
    /**
     * Runs a function with the app as active instance. This allows using of `inject()` within the function to get access
     * to variables provided via `app.provide()`.
     *
     * @param fn - function to run with the app as active instance
     */
    runWithContext<T>(fn: () => T): T;
    _uid: number;
    _component: ConcreteComponent;
    _props: Data | null;
    _container: HostElement | null;
    _context: AppContext;
    _instance: ComponentInternalInstance | null;
    /**
     * v2 compat only
     */
    filter?(name: string): Function | undefined;
    filter?(name: string, filter: Function): this;
}
export type OptionMergeFunction = (to: unknown, from: unknown) => any;
export interface AppConfig {
    readonly isNativeTag: (tag: string) => boolean;
    performance: boolean;
    optionMergeStrategies: Record<string, OptionMergeFunction>;
    globalProperties: ComponentCustomProperties & Record<string, any>;
    errorHandler?: (err: unknown, instance: ComponentPublicInstance | null, info: string) => void;
    warnHandler?: (msg: string, instance: ComponentPublicInstance | null, trace: string) => void;
    /**
     * Options to pass to `@vue/compiler-dom`.
     * Only supported in runtime compiler build.
     */
    compilerOptions: RuntimeCompilerOptions;
    /**
     * @deprecated use config.compilerOptions.isCustomElement
     */
    isCustomElement?: (tag: string) => boolean;
    /**
     * TODO document for 3.5
     * Enable warnings for computed getters that recursively trigger itself.
     */
    warnRecursiveComputed?: boolean;
    /**
     * Whether to throw unhandled errors in production.
     * Default is `false` to avoid crashing on any error (and only logs it)
     * But in some cases, e.g. SSR, throwing might be more desirable.
     */
    throwUnhandledErrorInProduction?: boolean;
    /**
     * Prefix for all useId() calls within this app
     */
    idPrefix?: string;
}
export interface AppContext {
    app: App;
    config: AppConfig;
    mixins: ComponentOptions[];
    components: Record<string, Component>;
    directives: Record<string, Directive>;
    provides: Record<string | symbol, any>;
}
type PluginInstallFunction<Options = any[]> = Options extends unknown[] ? (app: App, ...options: Options) => any : (app: App, options: Options) => any;
export type ObjectPlugin<Options = any[]> = {
    install: PluginInstallFunction<Options>;
};
export type FunctionPlugin<Options = any[]> = PluginInstallFunction<Options> & Partial<ObjectPlugin<Options>>;
export type Plugin<Options = any[], P extends unknown[] = Options extends unknown[] ? Options : [Options]> = FunctionPlugin<P> | ObjectPlugin<P>;
export declare function createAppContext(): AppContext;
export type CreateAppFunction<HostElement> = (rootComponent: Component, rootProps?: Data | null) => App<HostElement>;
export declare function createAppAPI<HostElement>(render: RootRenderFunction<HostElement>, hydrate?: RootHydrateFunction): CreateAppFunction<HostElement>;
export {};
