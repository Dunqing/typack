import { type ComponentInternalInstance } from './component';
import { type VNode, type VNodeArrayChildren } from './vnode';
export declare function markAttrsAccessed(): void;
export declare function renderComponentRoot(instance: ComponentInternalInstance): VNode;
export declare function filterSingleRoot(children: VNodeArrayChildren, recurse?: boolean): VNode | undefined;
export declare function shouldUpdateComponent(prevVNode: VNode, nextVNode: VNode, optimized?: boolean): boolean;
export declare function updateHOCHostEl({ vnode, parent }: ComponentInternalInstance, el: typeof vnode.el): void;
