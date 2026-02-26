import { type ComponentInternalInstance } from '../component';
import { type VNode, type VNodeProps } from '../vnode';
import { type ElementNamespace, type RendererElement, type RendererInternals, type RendererNode } from '../renderer';
import type { ComponentRenderContext } from '../componentPublicInstance';
type MatchPattern = string | RegExp | (string | RegExp)[];
export interface KeepAliveProps {
    include?: MatchPattern;
    exclude?: MatchPattern;
    max?: number | string;
}
export interface KeepAliveContext extends ComponentRenderContext {
    renderer: RendererInternals;
    activate: (vnode: VNode, container: RendererElement, anchor: RendererNode | null, namespace: ElementNamespace, optimized: boolean) => void;
    deactivate: (vnode: VNode) => void;
}
export declare const isKeepAlive: (vnode: VNode) => boolean;
export declare const KeepAlive: {
    __isKeepAlive: true;
    new (): {
        $props: VNodeProps & KeepAliveProps;
        $slots: {
            default(): VNode[];
        };
    };
};
export declare function onActivated(hook: Function, target?: ComponentInternalInstance | null): void;
export declare function onDeactivated(hook: Function, target?: ComponentInternalInstance | null): void;
export {};
