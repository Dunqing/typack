import type { SuspenseBoundary } from './components/Suspense';
import type { VNode, VNodeNormalizedRef } from './vnode';
/**
 * Function for handling a template ref
 */
export declare function setRef(rawRef: VNodeNormalizedRef, oldRawRef: VNodeNormalizedRef | null, parentSuspense: SuspenseBoundary | null, vnode: VNode, isUnmount?: boolean): void;
