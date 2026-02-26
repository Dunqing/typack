import { type Component, type ComponentInternalInstance } from './component';
import type { ComponentPublicInstance } from './componentPublicInstance';
import { type VNode } from './vnode';
import { type HydrationStrategy } from './hydrationStrategies';
export type AsyncComponentResolveResult<T = Component> = T | {
    default: T;
};
export type AsyncComponentLoader<T = any> = () => Promise<AsyncComponentResolveResult<T>>;
export interface AsyncComponentOptions<T = any> {
    loader: AsyncComponentLoader<T>;
    loadingComponent?: Component;
    errorComponent?: Component;
    delay?: number;
    timeout?: number;
    suspensible?: boolean;
    hydrate?: HydrationStrategy;
    onError?: (error: Error, retry: () => void, fail: () => void, attempts: number) => any;
}
export declare const isAsyncWrapper: (i: ComponentInternalInstance | VNode) => boolean;
export declare function defineAsyncComponent<T extends Component = {
    new (): ComponentPublicInstance;
}>(source: AsyncComponentLoader<T> | AsyncComponentOptions<T>): T;
