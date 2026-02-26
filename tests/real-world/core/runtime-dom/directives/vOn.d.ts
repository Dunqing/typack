import { type Directive } from '@vue/runtime-core';
declare const systemModifiers: readonly ["ctrl", "shift", "alt", "meta"];
type SystemModifiers = (typeof systemModifiers)[number];
type CompatModifiers = keyof typeof keyNames;
export type VOnModifiers = SystemModifiers | ModifierGuards | CompatModifiers;
type ModifierGuards = 'shift' | 'ctrl' | 'alt' | 'meta' | 'left' | 'right' | 'stop' | 'prevent' | 'self' | 'middle' | 'exact';
/**
 * @private
 */
export declare const withModifiers: <T extends (event: Event, ...args: unknown[]) => any>(fn: T & {
    _withMods?: {
        [key: string]: T;
    };
}, modifiers: VOnModifiers[]) => T;
declare const keyNames: Record<'esc' | 'space' | 'up' | 'left' | 'right' | 'down' | 'delete', string>;
/**
 * @private
 */
export declare const withKeys: <T extends (event: KeyboardEvent) => any>(fn: T & {
    _withKeys?: {
        [k: string]: T;
    };
}, modifiers: string[]) => T;
export type VOnDirective = Directive<any, any, VOnModifiers>;
export {};
