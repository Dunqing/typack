import { type ComponentInternalInstance } from '@vue/runtime-core';
interface Invoker extends EventListener {
    value: EventValue;
    attached: number;
}
type EventValue = Function | Function[];
export declare function addEventListener(el: Element, event: string, handler: EventListener, options?: EventListenerOptions): void;
export declare function removeEventListener(el: Element, event: string, handler: EventListener, options?: EventListenerOptions): void;
declare const veiKey: unique symbol;
export declare function patchEvent(el: Element & {
    [veiKey]?: Record<string, Invoker | undefined>;
}, rawName: string, prevValue: EventValue | null, nextValue: EventValue | unknown, instance?: ComponentInternalInstance | null): void;
export {};
