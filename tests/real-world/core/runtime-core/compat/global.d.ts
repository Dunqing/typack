import { reactive } from '@vue/reactivity';
import type { RootRenderFunction } from '../renderer';
import type { App, AppConfig, AppContext, CreateAppFunction, Plugin } from '../apiCreateApp';
import { type Component, type ComponentOptions } from '../component';
import { type RenderFunction } from '../componentOptions';
import type { Directive } from '../directives';
import { nextTick } from '../scheduler';
import { type LegacyConfig } from './globalConfig';
import { configureCompat } from './compatConfig';
import type { LegacyPublicInstance } from './instance';
/**
 * @deprecated the default `Vue` export has been removed in Vue 3. The type for
 * the default export is provided only for migration purposes. Please use
 * named imports instead - e.g. `import { createApp } from 'vue'`.
 */
export type CompatVue = Pick<App, 'version' | 'component' | 'directive'> & {
    configureCompat: typeof configureCompat;
    new (options?: ComponentOptions): LegacyPublicInstance;
    version: string;
    config: AppConfig & LegacyConfig;
    nextTick: typeof nextTick;
    use<Options extends unknown[]>(plugin: Plugin<Options>, ...options: Options): CompatVue;
    use<Options>(plugin: Plugin<Options>, options: Options): CompatVue;
    mixin(mixin: ComponentOptions): CompatVue;
    component(name: string): Component | undefined;
    component(name: string, component: Component): CompatVue;
    directive<T = any, V = any>(name: string): Directive<T, V> | undefined;
    directive<T = any, V = any>(name: string, directive: Directive<T, V>): CompatVue;
    compile(template: string): RenderFunction;
    /**
     * @deprecated Vue 3 no longer supports extending constructors.
     */
    extend: (options?: ComponentOptions) => CompatVue;
    /**
     * @deprecated Vue 3 no longer needs set() for adding new properties.
     */
    set(target: any, key: PropertyKey, value: any): void;
    /**
     * @deprecated Vue 3 no longer needs delete() for property deletions.
     */
    delete(target: any, key: PropertyKey): void;
    /**
     * @deprecated use `reactive` instead.
     */
    observable: typeof reactive;
    /**
     * @deprecated filters have been removed from Vue 3.
     */
    filter(name: string, arg?: any): null;
};
export declare let isCopyingConfig: boolean;
export declare let singletonApp: App;
export declare function createCompatVue(createApp: CreateAppFunction<Element>, createSingletonApp: CreateAppFunction<Element>): CompatVue;
export declare function installAppCompatProperties(app: App, context: AppContext, render: RootRenderFunction<any>): void;
