import { type VNode, type VNodeProps } from '../vnode';
import { type ComponentInternalInstance } from '../component';
import { type ElementNamespace, MoveType, type RendererElement, type RendererInternals, type RendererNode, type SetupRenderEffectFn } from '../renderer';
export interface SuspenseProps {
    onResolve?: () => void;
    onPending?: () => void;
    onFallback?: () => void;
    timeout?: string | number;
    /**
     * Allow suspense to be captured by parent suspense
     *
     * @default false
     */
    suspensible?: boolean;
}
export declare const isSuspense: (type: any) => boolean;
/**
 * For testing only
 */
export declare const resetSuspenseId: () => number;
export declare const SuspenseImpl: {
    name: string;
    __isSuspense: boolean;
    process(n1: VNode | null, n2: VNode, container: RendererElement, anchor: RendererNode | null, parentComponent: ComponentInternalInstance | null, parentSuspense: SuspenseBoundary | null, namespace: ElementNamespace, slotScopeIds: string[] | null, optimized: boolean, rendererInternals: RendererInternals): void;
    hydrate: typeof hydrateSuspense;
    normalize: typeof normalizeSuspenseChildren;
};
export declare const Suspense: {
    __isSuspense: true;
    new (): {
        $props: VNodeProps & SuspenseProps;
        $slots: {
            default(): VNode[];
            fallback(): VNode[];
        };
    };
};
export interface SuspenseBoundary {
    vnode: VNode<RendererNode, RendererElement, SuspenseProps>;
    parent: SuspenseBoundary | null;
    parentComponent: ComponentInternalInstance | null;
    namespace: ElementNamespace;
    container: RendererElement;
    hiddenContainer: RendererElement;
    activeBranch: VNode | null;
    pendingBranch: VNode | null;
    deps: number;
    pendingId: number;
    timeout: number;
    isInFallback: boolean;
    isHydrating: boolean;
    isUnmounted: boolean;
    effects: Function[];
    resolve(force?: boolean, sync?: boolean): void;
    fallback(fallbackVNode: VNode): void;
    move(container: RendererElement, anchor: RendererNode | null, type: MoveType): void;
    next(): RendererNode | null;
    registerDep(instance: ComponentInternalInstance, setupRenderEffect: SetupRenderEffectFn, optimized: boolean): void;
    unmount(parentSuspense: SuspenseBoundary | null, doRemove?: boolean): void;
}
declare function hydrateSuspense(node: Node, vnode: VNode, parentComponent: ComponentInternalInstance | null, parentSuspense: SuspenseBoundary | null, namespace: ElementNamespace, slotScopeIds: string[] | null, optimized: boolean, rendererInternals: RendererInternals, hydrateNode: (node: Node, vnode: VNode, parentComponent: ComponentInternalInstance | null, parentSuspense: SuspenseBoundary | null, slotScopeIds: string[] | null, optimized: boolean) => Node | null): Node | null;
declare function normalizeSuspenseChildren(vnode: VNode): void;
export declare function queueEffectWithSuspense(fn: Function | Function[], suspense: SuspenseBoundary | null): void;
export {};
