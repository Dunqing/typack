import { type ComponentInternalInstance, type VNode, type VNodeArrayChildren } from 'vue';
export type SSRBuffer = SSRBufferItem[] & {
    hasAsync?: boolean;
};
export type SSRBufferItem = string | SSRBuffer | Promise<SSRBuffer>;
export type PushFn = (item: SSRBufferItem) => void;
export type Props = Record<string, unknown>;
export type SSRContext = {
    [key: string]: any;
    teleports?: Record<string, string>;
};
export declare function createBuffer(): {
    getBuffer(): SSRBuffer;
    push(item: SSRBufferItem): void;
};
export declare function renderComponentVNode(vnode: VNode, parentComponent?: ComponentInternalInstance | null, slotScopeId?: string): SSRBuffer | Promise<SSRBuffer>;
export declare function renderVNode(push: PushFn, vnode: VNode, parentComponent: ComponentInternalInstance, slotScopeId?: string): void;
export declare function renderVNodeChildren(push: PushFn, children: VNodeArrayChildren, parentComponent: ComponentInternalInstance, slotScopeId?: string): void;
