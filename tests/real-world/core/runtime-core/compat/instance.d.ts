import type { ComponentPublicInstance, PublicPropertiesMap } from '../componentPublicInstance';
import type { Slots } from '../componentSlots';
export type LegacyPublicInstance = ComponentPublicInstance & LegacyPublicProperties;
export interface LegacyPublicProperties {
    $set<T extends Record<keyof any, any>, K extends keyof T>(target: T, key: K, value: T[K]): void;
    $delete<T extends Record<keyof any, any>, K extends keyof T>(target: T, key: K): void;
    $mount(el?: string | Element): this;
    $destroy(): void;
    $scopedSlots: Slots;
    $on(event: string | string[], fn: Function): this;
    $once(event: string, fn: Function): this;
    $off(event?: string | string[], fn?: Function): this;
    $children: LegacyPublicProperties[];
    $listeners: Record<string, Function | Function[]>;
}
export declare function installCompatInstanceProperties(map: PublicPropertiesMap): void;
