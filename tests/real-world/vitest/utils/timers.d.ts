export interface SafeTimers {
    nextTick?: (cb: () => void) => void;
    setImmediate?: {
        <TArgs extends any[]>(callback: (...args: TArgs) => void, ...args: TArgs): any;
        __promisify__: <T = void>(value?: T, options?: any) => Promise<T>;
    };
    clearImmediate?: (immediateId: any) => void;
    setTimeout: typeof setTimeout;
    setInterval: typeof setInterval;
    clearInterval: typeof clearInterval;
    clearTimeout: typeof clearTimeout;
    queueMicrotask: typeof queueMicrotask;
}
export declare function getSafeTimers(): SafeTimers;
export declare function setSafeTimers(): void;
/**
 * Returns a promise that resolves after the specified duration.
 *
 * @param timeout - Delay in milliseconds
 * @param scheduler - Timer function to use, defaults to `setTimeout`. Useful for mocked timers.
 *
 * @example
 * await delay(100)
 *
 * @example
 * // With mocked timers
 * const { setTimeout } = getSafeTimers()
 * await delay(100, setTimeout)
 */
export declare function delay(timeout: number, scheduler?: typeof setTimeout): Promise<void>;
