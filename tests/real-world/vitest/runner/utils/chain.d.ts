import type { InternalChainableContext, SuiteAPI, TestAPI } from '../types/tasks';
export type ChainableFunction<T extends string, F extends (...args: any) => any, C = object> = F & {
    [x in T]: ChainableFunction<T, F, C>;
} & {
    fn: (this: Record<T, any>, ...args: Parameters<F>) => ReturnType<F>;
} & C;
export declare const kChainableContext: unique symbol;
export declare function getChainableContext(chainable: SuiteAPI): InternalChainableContext;
export declare function getChainableContext(chainable: TestAPI): InternalChainableContext;
export declare function getChainableContext(chainable: any): InternalChainableContext | undefined;
export declare function createChainable<T extends string, Args extends any[], R = any>(keys: T[], fn: (this: Record<T, any>, ...args: Args) => R, context?: Record<string, any>): ChainableFunction<T, (...args: Args) => R>;
