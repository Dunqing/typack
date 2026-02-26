import { type VNode } from '../vnode';
export declare function withMemo(memo: any[], render: () => VNode<any, any>, cache: any[], index: number): VNode<any, any>;
export declare function isMemoSame(cached: VNode, memo: any[]): boolean;
