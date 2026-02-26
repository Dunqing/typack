import type { ComponentInternalInstance } from '../component';
import type { VNode } from '../vnode';
export declare const compatModelEventPrefix = "onModelCompat:";
export declare function convertLegacyVModelProps(vnode: VNode): void;
export declare function compatModelEmit(instance: ComponentInternalInstance, event: string, args: any[]): void;
