import { type BaseTransitionProps, type FunctionalComponent } from '@vue/runtime-core';
declare const TRANSITION = "transition";
declare const ANIMATION = "animation";
type AnimationTypes = typeof TRANSITION | typeof ANIMATION;
export interface TransitionProps extends BaseTransitionProps<Element> {
    name?: string;
    type?: AnimationTypes;
    css?: boolean;
    duration?: number | {
        enter: number;
        leave: number;
    };
    enterFromClass?: string;
    enterActiveClass?: string;
    enterToClass?: string;
    appearFromClass?: string;
    appearActiveClass?: string;
    appearToClass?: string;
    leaveFromClass?: string;
    leaveActiveClass?: string;
    leaveToClass?: string;
}
export declare const vtcKey: unique symbol;
export interface ElementWithTransition extends HTMLElement {
    [vtcKey]?: Set<string>;
}
export declare const TransitionPropsValidators: any;
/**
 * DOM Transition is a higher-order-component based on the platform-agnostic
 * base Transition component, with DOM-specific logic.
 */
export declare const Transition: FunctionalComponent<TransitionProps>;
export declare function resolveTransitionProps(rawProps: TransitionProps): BaseTransitionProps<Element>;
export declare function addTransitionClass(el: Element, cls: string): void;
export declare function removeTransitionClass(el: Element, cls: string): void;
interface CSSTransitionInfo {
    type: AnimationTypes | null;
    propCount: number;
    timeout: number;
    hasTransform: boolean;
}
export declare function getTransitionInfo(el: Element, expectedType?: TransitionProps['type']): CSSTransitionInfo;
export declare function forceReflow(el?: Node): number;
export {};
