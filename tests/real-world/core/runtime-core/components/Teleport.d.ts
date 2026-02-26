import type { ComponentInternalInstance } from '../component';
import type { SuspenseBoundary } from './Suspense';
import { type ElementNamespace, type RendererElement, type RendererInternals, type RendererNode } from '../renderer';
import type { VNode, VNodeProps } from '../vnode';
export type TeleportVNode = VNode<RendererNode, RendererElement, TeleportProps>;
export interface TeleportProps {
    to: string | RendererElement | null | undefined;
    disabled?: boolean;
    defer?: boolean;
}
export declare const TeleportEndKey: unique symbol;
export declare const isTeleport: (type: any) => boolean;
export declare const TeleportImpl: {
    name: string;
    __isTeleport: boolean;
    process(n1: TeleportVNode | null, n2: TeleportVNode, container: RendererElement, anchor: RendererNode | null, parentComponent: ComponentInternalInstance | null, parentSuspense: SuspenseBoundary | null, namespace: ElementNamespace, slotScopeIds: string[] | null, optimized: boolean, internals: RendererInternals): void;
    remove(vnode: VNode, parentComponent: ComponentInternalInstance | null, parentSuspense: SuspenseBoundary | null, { um: unmount, o: { remove: hostRemove } }: RendererInternals, doRemove: boolean): void;
    move: typeof moveTeleport;
    hydrate: typeof hydrateTeleport;
};
export declare enum TeleportMoveTypes {
    TARGET_CHANGE = 0,
    TOGGLE = 1,// enable / disable
    REORDER = 2
}
declare function moveTeleport(vnode: VNode, container: RendererElement, parentAnchor: RendererNode | null, { o: { insert }, m: move }: RendererInternals, moveType?: TeleportMoveTypes): void;
declare function hydrateTeleport(node: Node, vnode: TeleportVNode, parentComponent: ComponentInternalInstance | null, parentSuspense: SuspenseBoundary | null, slotScopeIds: string[] | null, optimized: boolean, { o: { nextSibling, parentNode, querySelector, insert, createText }, }: RendererInternals<Node, Element>, hydrateChildren: (node: Node | null, vnode: VNode, container: Element, parentComponent: ComponentInternalInstance | null, parentSuspense: SuspenseBoundary | null, slotScopeIds: string[] | null, optimized: boolean) => Node | null): Node | null;
export declare const Teleport: {
    __isTeleport: true;
    new (): {
        $props: VNodeProps & TeleportProps;
        $slots: {
            default(): VNode[];
        };
    };
};
export {};
