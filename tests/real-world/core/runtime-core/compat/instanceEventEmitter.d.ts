import type { ComponentInternalInstance } from '../component';
import type { ComponentPublicInstance } from '../componentPublicInstance';
interface EventRegistry {
    [event: string]: Function[] | undefined;
}
export declare function getRegistry(instance: ComponentInternalInstance): EventRegistry;
export declare function on(instance: ComponentInternalInstance, event: string | string[], fn: Function): ComponentPublicInstance | null;
export declare function once(instance: ComponentInternalInstance, event: string, fn: Function): ComponentPublicInstance | null;
export declare function off(instance: ComponentInternalInstance, event?: string | string[], fn?: Function): ComponentPublicInstance | null;
export declare function emit(instance: ComponentInternalInstance, event: string, args: any[]): ComponentPublicInstance | null;
export {};
