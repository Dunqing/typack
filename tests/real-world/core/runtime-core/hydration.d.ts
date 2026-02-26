import { type VNode } from './vnode';
import type { ComponentInternalInstance } from './component';
import { type RendererInternals } from './renderer';
import { type SuspenseBoundary } from './components/Suspense';
export type RootHydrateFunction = (vnode: VNode<Node, Element>, container: (Element | ShadowRoot) & {
    _vnode?: VNode;
}) => void;
export declare enum DOMNodeTypes {
    ELEMENT = 1,
    TEXT = 3,
    COMMENT = 8
}
export declare const isComment: (node: Node) => node is Comment;
export declare function createHydrationFunctions(rendererInternals: RendererInternals<Node, Element>): [
    RootHydrateFunction,
    (node: Node, vnode: VNode, parentComponent: ComponentInternalInstance | null, parentSuspense: SuspenseBoundary | null, slotScopeIds: string[] | null, optimized?: boolean) => Node | null
];
