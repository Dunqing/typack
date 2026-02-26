export interface ConcurrencyLimiter extends ConcurrencyLimiterFn {
    acquire: () => (() => void) | Promise<() => void>;
}
type ConcurrencyLimiterFn = <Args extends unknown[], T>(func: (...args: Args) => PromiseLike<T> | T, ...args: Args) => Promise<T>;
/**
 * Return a function for running multiple async operations with limited concurrency.
 */
export declare function limitConcurrency(concurrency?: number): ConcurrencyLimiter;
export {};
