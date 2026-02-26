import { type CreateAppFunction, type DefineComponent, type Directive, type RootHydrateFunction, type RootRenderFunction } from '@vue/runtime-core';
import { nodeOps } from './runtime-dom/nodeOps';
import { patchProp } from './runtime-dom/patchProp';
export { nodeOps, patchProp };
import type { TransitionProps } from './runtime-dom/components/Transition';
import type { TransitionGroupProps } from './runtime-dom/components/TransitionGroup';
import type { vShow } from './runtime-dom/directives/vShow';
import type { VOnDirective } from './runtime-dom/directives/vOn';
import type { VModelDirective } from './runtime-dom/directives/vModel';
/**
 * This is a stub implementation to prevent the need to use dom types.
 *
 * To enable proper types, add `"dom"` to `"lib"` in your `tsconfig.json`.
 */
type DomType<T> = typeof globalThis extends {
    window: unknown;
} ? T : never;
declare module '@vue/reactivity' {
    interface RefUnwrapBailTypes {
        runtimeDOMBailTypes: DomType<Node | Window>;
    }
}
declare module '@vue/runtime-core' {
    interface GlobalComponents {
        Transition: DefineComponent<TransitionProps>;
        TransitionGroup: DefineComponent<TransitionGroupProps>;
    }
    interface GlobalDirectives {
        vShow: typeof vShow;
        vOn: VOnDirective;
        vBind: VModelDirective;
        vIf: Directive<any, boolean>;
        vOnce: Directive;
        vSlot: Directive;
    }
}
export declare const render: RootRenderFunction<Element | ShadowRoot>;
export declare const hydrate: RootHydrateFunction;
export declare const createApp: CreateAppFunction<Element>;
export declare const createSSRApp: CreateAppFunction<Element>;
export { defineCustomElement, defineSSRCustomElement, useShadowRoot, useHost, VueElement, type VueElementConstructor, type CustomElementOptions, } from './runtime-dom/apiCustomElement';
export { useCssModule } from './runtime-dom/helpers/useCssModule';
export { useCssVars } from './runtime-dom/helpers/useCssVars';
export { Transition, type TransitionProps } from './runtime-dom/components/Transition';
export { TransitionGroup, type TransitionGroupProps, } from './runtime-dom/components/TransitionGroup';
export { vModelText, vModelCheckbox, vModelRadio, vModelSelect, vModelDynamic, } from './runtime-dom/directives/vModel';
export { withModifiers, withKeys } from './runtime-dom/directives/vOn';
export { vShow } from './runtime-dom/directives/vShow';
export * from '@vue/runtime-core';
export * from './runtime-dom/jsx';
