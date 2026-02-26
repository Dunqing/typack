import { type ComponentInternalInstance } from '../component';
import { type VNode } from '../vnode';
import type { RendererElement } from '../renderer';
type Hook<T = () => void> = T | T[];
export declare const leaveCbKey: unique symbol;
declare const enterCbKey: unique symbol;
export interface BaseTransitionProps<HostElement = RendererElement> {
    mode?: 'in-out' | 'out-in' | 'default';
    appear?: boolean;
    persisted?: boolean;
    onBeforeEnter?: Hook<(el: HostElement) => void>;
    onEnter?: Hook<(el: HostElement, done: () => void) => void>;
    onAfterEnter?: Hook<(el: HostElement) => void>;
    onEnterCancelled?: Hook<(el: HostElement) => void>;
    onBeforeLeave?: Hook<(el: HostElement) => void>;
    onLeave?: Hook<(el: HostElement, done: () => void) => void>;
    onAfterLeave?: Hook<(el: HostElement) => void>;
    onLeaveCancelled?: Hook<(el: HostElement) => void>;
    onBeforeAppear?: Hook<(el: HostElement) => void>;
    onAppear?: Hook<(el: HostElement, done: () => void) => void>;
    onAfterAppear?: Hook<(el: HostElement) => void>;
    onAppearCancelled?: Hook<(el: HostElement) => void>;
}
export interface TransitionHooks<HostElement = RendererElement> {
    mode: BaseTransitionProps['mode'];
    persisted: boolean;
    beforeEnter(el: HostElement): void;
    enter(el: HostElement): void;
    leave(el: HostElement, remove: () => void): void;
    clone(vnode: VNode): TransitionHooks<HostElement>;
    afterLeave?(): void;
    delayLeave?(el: HostElement, earlyRemove: () => void, delayedLeave: () => void): void;
    delayedLeave?(): void;
}
export type TransitionHookCaller = <T extends any[] = [el: any]>(hook: Hook<(...args: T) => void> | undefined, args?: T) => void;
export type PendingCallback = (cancelled?: boolean) => void;
export interface TransitionState {
    isMounted: boolean;
    isLeaving: boolean;
    isUnmounting: boolean;
    leavingVNodes: Map<any, Record<string, VNode>>;
}
export interface TransitionElement {
    [enterCbKey]?: PendingCallback;
    [leaveCbKey]?: PendingCallback;
}
export declare function useTransitionState(): TransitionState;
export declare const BaseTransitionPropsValidators: Record<string, any>;
export declare const BaseTransition: {
    new (): {
        $props: BaseTransitionProps<any>;
        $slots: {
            default(): VNode[];
        };
    };
};
export declare function resolveTransitionHooks(vnode: VNode, props: BaseTransitionProps<any>, state: TransitionState, instance: ComponentInternalInstance, postClone?: (hooks: TransitionHooks) => void): TransitionHooks;
export declare function setTransitionHooks(vnode: VNode, hooks: TransitionHooks): void;
export declare function getTransitionRawChildren(children: VNode[], keepComment?: boolean, parentKey?: VNode['key']): VNode[];
export {};
