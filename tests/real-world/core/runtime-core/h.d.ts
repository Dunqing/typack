import { type Comment, type Fragment, type Text, type VNode, type VNodeArrayChildren, type VNodeProps } from './vnode';
import type { Teleport, TeleportProps } from './components/Teleport';
import type { Suspense, SuspenseProps } from './components/Suspense';
import { type IfAny } from '@vue/shared';
import type { RawSlots } from './componentSlots';
import type { Component, ComponentOptions, ConcreteComponent, FunctionalComponent } from './component';
import type { EmitsOptions } from './componentEmits';
import type { DefineComponent } from './apiDefineComponent';
type RawProps = VNodeProps & {
    __v_isVNode?: never;
    [Symbol.iterator]?: never;
} & Record<string, any>;
type RawChildren = string | number | boolean | VNode | VNodeArrayChildren | (() => any);
interface Constructor<P = any> {
    __isFragment?: never;
    __isTeleport?: never;
    __isSuspense?: never;
    new (...args: any[]): {
        $props: P;
    };
}
type HTMLElementEventHandler = {
    [K in keyof HTMLElementEventMap as `on${Capitalize<K>}`]?: (ev: HTMLElementEventMap[K]) => any;
};
export declare function h<K extends keyof HTMLElementTagNameMap>(type: K, children?: RawChildren): VNode;
export declare function h<K extends keyof HTMLElementTagNameMap>(type: K, props?: (RawProps & HTMLElementEventHandler) | null, children?: RawChildren | RawSlots): VNode;
export declare function h(type: string, children?: RawChildren): VNode;
export declare function h(type: string, props?: RawProps | null, children?: RawChildren | RawSlots): VNode;
export declare function h(type: typeof Text | typeof Comment, children?: string | number | boolean): VNode;
export declare function h(type: typeof Text | typeof Comment, props?: null, children?: string | number | boolean): VNode;
export declare function h(type: typeof Fragment, children?: VNodeArrayChildren): VNode;
export declare function h(type: typeof Fragment, props?: RawProps | null, children?: VNodeArrayChildren): VNode;
export declare function h(type: typeof Teleport, props: RawProps & TeleportProps, children: RawChildren | RawSlots): VNode;
export declare function h(type: typeof Suspense, children?: RawChildren): VNode;
export declare function h(type: typeof Suspense, props?: (RawProps & SuspenseProps) | null, children?: RawChildren | RawSlots): VNode;
export declare function h<P, E extends EmitsOptions = {}, S extends Record<string, any> = any>(type: FunctionalComponent<P, any, S, any>, props?: (RawProps & P) | ({} extends P ? null : never), children?: RawChildren | IfAny<S, RawSlots, S>): VNode;
export declare function h(type: Component, children?: RawChildren): VNode;
export declare function h<P>(type: ConcreteComponent | string, children?: RawChildren): VNode;
export declare function h<P>(type: ConcreteComponent<P> | string, props?: (RawProps & P) | ({} extends P ? null : never), children?: RawChildren): VNode;
export declare function h<P>(type: Component<P>, props?: (RawProps & P) | null, children?: RawChildren | RawSlots): VNode;
export declare function h<P>(type: ComponentOptions<P>, props?: (RawProps & P) | ({} extends P ? null : never), children?: RawChildren | RawSlots): VNode;
export declare function h(type: Constructor, children?: RawChildren): VNode;
export declare function h<P>(type: Constructor<P>, props?: (RawProps & P) | ({} extends P ? null : never), children?: RawChildren | RawSlots): VNode;
export declare function h(type: DefineComponent, children?: RawChildren): VNode;
export declare function h<P>(type: DefineComponent<P>, props?: (RawProps & P) | ({} extends P ? null : never), children?: RawChildren | RawSlots): VNode;
export declare function h(type: string | Component, children?: RawChildren): VNode;
export declare function h<P>(type: string | Component<P>, props?: (RawProps & P) | ({} extends P ? null : never), children?: RawChildren | RawSlots): VNode;
export {};
