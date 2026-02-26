import { type ObjectDirective } from '@vue/runtime-core';
type AssignerFn = (value: any) => void;
declare const assignKey: unique symbol;
type ModelDirective<T, Modifiers extends string = string> = ObjectDirective<T & {
    [assignKey]: AssignerFn;
    _assigning?: boolean;
}, any, Modifiers>;
export declare const vModelText: ModelDirective<HTMLInputElement | HTMLTextAreaElement, 'trim' | 'number' | 'lazy'>;
export declare const vModelCheckbox: ModelDirective<HTMLInputElement>;
export declare const vModelRadio: ModelDirective<HTMLInputElement>;
export declare const vModelSelect: ModelDirective<HTMLSelectElement, 'number'>;
export declare const vModelDynamic: ObjectDirective<HTMLInputElement | HTMLSelectElement | HTMLTextAreaElement>;
export declare function initVModelForSSR(): void;
export type VModelDirective = typeof vModelText | typeof vModelCheckbox | typeof vModelSelect | typeof vModelRadio | typeof vModelDynamic;
export {};
