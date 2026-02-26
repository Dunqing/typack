/**
 * A lazy hydration strategy for async components.
 * @param hydrate - call this to perform the actual hydration.
 * @param forEachElement - iterate through the root elements of the component's
 *                         non-hydrated DOM, accounting for possible fragments.
 * @returns a teardown function to be called if the async component is unmounted
 *          before it is hydrated. This can be used to e.g. remove DOM event
 *          listeners.
 */
export type HydrationStrategy = (hydrate: () => void, forEachElement: (cb: (el: Element) => any) => void) => (() => void) | void;
export type HydrationStrategyFactory<Options> = (options?: Options) => HydrationStrategy;
export declare const hydrateOnIdle: HydrationStrategyFactory<number>;
export declare const hydrateOnVisible: HydrationStrategyFactory<IntersectionObserverInit>;
export declare const hydrateOnMediaQuery: HydrationStrategyFactory<string>;
export declare const hydrateOnInteraction: HydrationStrategyFactory<keyof HTMLElementEventMap | Array<keyof HTMLElementEventMap>>;
export declare function forEachElement(node: Node, cb: (el: Element) => void | false): void;
