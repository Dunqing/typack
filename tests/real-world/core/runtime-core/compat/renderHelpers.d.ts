import type { ComponentInternalInstance, Data } from '../component';
import type { Slot } from '../componentSlots';
import { createSlots } from '../helpers/createSlots';
import { type VNode } from '../vnode';
export declare function legacyBindObjectProps(data: any, _tag: string, value: any, _asProp: boolean, isSync?: boolean): any;
export declare function legacyBindObjectListeners(props: any, listeners: any): Data;
export declare function legacyRenderSlot(instance: ComponentInternalInstance, name: string, fallback?: VNode[], props?: any, bindObject?: any): VNode;
type LegacyScopedSlotsData = Array<{
    key: string;
    fn: Function;
} | LegacyScopedSlotsData>;
export declare function legacyResolveScopedSlots(fns: LegacyScopedSlotsData, raw?: Record<string, Slot>, hasDynamicKeys?: boolean): ReturnType<typeof createSlots>;
export declare function legacyRenderStatic(instance: ComponentInternalInstance, index: number): any;
export declare function legacyCheckKeyCodes(instance: ComponentInternalInstance, eventKeyCode: number, key: string, builtInKeyCode?: number | number[], eventKeyName?: string, builtInKeyName?: string | string[]): boolean | undefined;
export declare function legacyMarkOnce(tree: VNode): VNode;
export declare function legacyBindDynamicKeys(props: any, values: any[]): any;
export declare function legacyPrependModifier(value: any, symbol: string): any;
export {};
