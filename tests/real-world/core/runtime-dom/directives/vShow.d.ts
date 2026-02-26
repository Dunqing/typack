import type { ObjectDirective } from '@vue/runtime-core';
export declare const vShowOriginalDisplay: unique symbol;
export declare const vShowHidden: unique symbol;
export interface VShowElement extends HTMLElement {
    [vShowOriginalDisplay]: string;
    [vShowHidden]: boolean;
}
export declare const vShow: ObjectDirective<VShowElement> & {
    name: 'show';
};
export declare function initVShowForSSR(): void;
