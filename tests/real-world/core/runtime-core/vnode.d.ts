import { type ClassComponent, type Component, type ComponentInternalInstance, type Data } from './component';
import type { RawSlots } from './componentSlots';
import { type Ref } from '@vue/reactivity';
import type { AppContext } from './apiCreateApp';
import { type Suspense, type SuspenseBoundary, type SuspenseImpl } from './components/Suspense';
import type { DirectiveBinding } from './directives';
import { type TransitionHooks } from './components/BaseTransition';
import { type Teleport, type TeleportImpl } from './components/Teleport';
import type { RendererElement, RendererNode } from './renderer';
import { NULL_DYNAMIC_COMPONENT } from './helpers/resolveAssets';
import type { ComponentPublicInstance } from './componentPublicInstance';
export declare const Fragment: {
    __isFragment: true;
    new (): {
        $props: VNodeProps;
    };
};
export declare const Text: unique symbol;
export declare const Comment: unique symbol;
export declare const Static: unique symbol;
export type VNodeTypes = string | VNode | Component | typeof Text | typeof Static | typeof Comment | typeof Fragment | typeof Teleport | typeof TeleportImpl | typeof Suspense | typeof SuspenseImpl;
export type VNodeRef = string | Ref | ((ref: Element | ComponentPublicInstance | null, refs: Record<string, any>) => void);
export type VNodeNormalizedRefAtom = {
    /**
     * component instance
     */
    i: ComponentInternalInstance;
    /**
     * Actual ref
     */
    r: VNodeRef;
    /**
     * setup ref key
     */
    k?: string;
    /**
     * refInFor marker
     */
    f?: boolean;
};
export type VNodeNormalizedRef = VNodeNormalizedRefAtom | VNodeNormalizedRefAtom[];
type VNodeMountHook = (vnode: VNode) => void;
type VNodeUpdateHook = (vnode: VNode, oldVNode: VNode) => void;
export type VNodeHook = VNodeMountHook | VNodeUpdateHook | VNodeMountHook[] | VNodeUpdateHook[];
export type VNodeProps = {
    key?: PropertyKey;
    ref?: VNodeRef;
    ref_for?: boolean;
    ref_key?: string;
    onVnodeBeforeMount?: VNodeMountHook | VNodeMountHook[];
    onVnodeMounted?: VNodeMountHook | VNodeMountHook[];
    onVnodeBeforeUpdate?: VNodeUpdateHook | VNodeUpdateHook[];
    onVnodeUpdated?: VNodeUpdateHook | VNodeUpdateHook[];
    onVnodeBeforeUnmount?: VNodeMountHook | VNodeMountHook[];
    onVnodeUnmounted?: VNodeMountHook | VNodeMountHook[];
};
type VNodeChildAtom = VNode | string | number | boolean | null | undefined | void;
export type VNodeArrayChildren = Array<VNodeArrayChildren | VNodeChildAtom>;
export type VNodeChild = VNodeChildAtom | VNodeArrayChildren;
export type VNodeNormalizedChildren = string | VNodeArrayChildren | RawSlots | null;
export interface VNode<HostNode = RendererNode, HostElement = RendererElement, ExtraProps = {
    [key: string]: any;
}> {
    type: VNodeTypes;
    props: (VNodeProps & ExtraProps) | null;
    key: PropertyKey | null;
    ref: VNodeNormalizedRef | null;
    /**
     * SFC only. This is assigned on vnode creation using currentScopeId
     * which is set alongside currentRenderingInstance.
     */
    scopeId: string | null;
    children: VNodeNormalizedChildren;
    component: ComponentInternalInstance | null;
    dirs: DirectiveBinding[] | null;
    transition: TransitionHooks<HostElement> | null;
    el: HostNode | null;
    placeholder: HostNode | null;
    anchor: HostNode | null;
    target: HostElement | null;
    targetStart: HostNode | null;
    targetAnchor: HostNode | null;
    suspense: SuspenseBoundary | null;
    shapeFlag: number;
    patchFlag: number;
    appContext: AppContext | null;
}
export declare const blockStack: VNode['dynamicChildren'][];
export declare let currentBlock: VNode['dynamicChildren'];
/**
 * Open a block.
 * This must be called before `createBlock`. It cannot be part of `createBlock`
 * because the children of the block are evaluated before `createBlock` itself
 * is called. The generated code typically looks like this:
 *
 * ```js
 * function render() {
 *   return (openBlock(),createBlock('div', null, [...]))
 * }
 * ```
 * disableTracking is true when creating a v-for fragment block, since a v-for
 * fragment always diffs its children.
 *
 * @private
 */
export declare function openBlock(disableTracking?: boolean): void;
export declare function closeBlock(): void;
export declare let isBlockTreeEnabled: number;
/**
 * Block tracking sometimes needs to be disabled, for example during the
 * creation of a tree that needs to be cached by v-once. The compiler generates
 * code like this:
 *
 * ``` js
 * _cache[1] || (
 *   setBlockTracking(-1, true),
 *   _cache[1] = createVNode(...),
 *   setBlockTracking(1),
 *   _cache[1]
 * )
 * ```
 *
 * @private
 */
export declare function setBlockTracking(value: number, inVOnce?: boolean): void;
/**
 * @private
 */
export declare function createElementBlock(type: string | typeof Fragment, props?: Record<string, any> | null, children?: any, patchFlag?: number, dynamicProps?: string[], shapeFlag?: number): VNode;
/**
 * Create a block root vnode. Takes the same exact arguments as `createVNode`.
 * A block root keeps track of dynamic nodes within the block in the
 * `dynamicChildren` array.
 *
 * @private
 */
export declare function createBlock(type: VNodeTypes | ClassComponent, props?: Record<string, any> | null, children?: any, patchFlag?: number, dynamicProps?: string[]): VNode;
export declare function isVNode(value: any): value is VNode;
export declare function isSameVNodeType(n1: VNode, n2: VNode): boolean;
declare let vnodeArgsTransformer: ((args: Parameters<typeof _createVNode>, instance: ComponentInternalInstance | null) => Parameters<typeof _createVNode>) | undefined;
/**
 * Internal API for registering an arguments transform for createVNode
 * used for creating stubs in the test-utils
 * It is *internal* but needs to be exposed for test-utils to pick up proper
 * typings
 */
export declare function transformVNodeArgs(transformer?: typeof vnodeArgsTransformer): void;
declare function createBaseVNode(type: VNodeTypes | ClassComponent | typeof NULL_DYNAMIC_COMPONENT, props?: (Data & VNodeProps) | null, children?: unknown, patchFlag?: number, dynamicProps?: string[] | null, shapeFlag?: number, isBlockNode?: boolean, needFullChildrenNormalization?: boolean): VNode;
export { createBaseVNode as createElementVNode };
export declare const createVNode: typeof _createVNode;
declare function _createVNode(type: VNodeTypes | ClassComponent | typeof NULL_DYNAMIC_COMPONENT, props?: (Data & VNodeProps) | null, children?: unknown, patchFlag?: number, dynamicProps?: string[] | null, isBlockNode?: boolean): VNode;
export declare function guardReactiveProps(props: (Data & VNodeProps) | null): (Data & VNodeProps) | null;
export declare function cloneVNode<T, U>(vnode: VNode<T, U>, extraProps?: (Data & VNodeProps) | null, mergeRef?: boolean, cloneTransition?: boolean): VNode<T, U>;
/**
 * @private
 */
export declare function createTextVNode(text?: string, flag?: number): VNode;
/**
 * @private
 */
export declare function createStaticVNode(content: string, numberOfNodes: number): VNode;
/**
 * @private
 */
export declare function createCommentVNode(text?: string, asBlock?: boolean): VNode;
export declare function normalizeVNode(child: VNodeChild): VNode;
export declare function cloneIfMounted(child: VNode): VNode;
export declare function normalizeChildren(vnode: VNode, children: unknown): void;
export declare function mergeProps(...args: (Data & VNodeProps)[]): Data;
export declare function invokeVNodeHook(hook: VNodeHook, instance: ComponentInternalInstance | null, vnode: VNode, prevVNode?: VNode | null): void;
