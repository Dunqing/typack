import type { Component, ComponentInternalInstance, Data } from '../component';
import { type VNode, type VNodeArrayChildren } from '../vnode';
export declare function convertLegacyRenderFn(instance: ComponentInternalInstance): void;
interface LegacyVNodeProps {
    key?: string | number;
    ref?: string;
    refInFor?: boolean;
    staticClass?: string;
    class?: unknown;
    staticStyle?: Record<string, unknown>;
    style?: Record<string, unknown>;
    attrs?: Record<string, unknown>;
    domProps?: Record<string, unknown>;
    on?: Record<string, Function | Function[]>;
    nativeOn?: Record<string, Function | Function[]>;
    directives?: LegacyVNodeDirective[];
    props?: Record<string, unknown>;
    slot?: string;
    scopedSlots?: Record<string, Function>;
    model?: {
        value: any;
        callback: (v: any) => void;
        expression: string;
    };
}
interface LegacyVNodeDirective {
    name: string;
    value: unknown;
    arg?: string;
    modifiers?: Record<string, boolean>;
}
type LegacyVNodeChildren = string | number | boolean | VNode | VNodeArrayChildren;
export declare function compatH(type: string | Component, children?: LegacyVNodeChildren): VNode;
export declare function compatH(type: string | Component, props?: Data & LegacyVNodeProps, children?: LegacyVNodeChildren): VNode;
export declare function defineLegacyVNodeProperties(vnode: VNode): void;
export {};
