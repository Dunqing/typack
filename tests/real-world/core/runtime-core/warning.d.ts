import type { VNode } from './vnode';
import { type ConcreteComponent } from './component';
type ComponentVNode = VNode & {
    type: ConcreteComponent;
};
type TraceEntry = {
    vnode: ComponentVNode;
    recurseCount: number;
};
type ComponentTraceStack = TraceEntry[];
export declare function pushWarningContext(vnode: VNode): void;
export declare function popWarningContext(): void;
export declare function warn(msg: string, ...args: any[]): void;
export declare function getComponentTrace(): ComponentTraceStack;
export {};
