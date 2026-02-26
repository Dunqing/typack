/**
 * Track array iteration and return:
 * - if input is reactive: a cloned raw array with reactive values
 * - if input is non-reactive or shallowReactive: the original raw array
 */
export declare function reactiveReadArray<T>(array: T[]): T[];
/**
 * Track array iteration and return raw array
 */
export declare function shallowReadArray<T>(arr: T[]): T[];
export declare const arrayInstrumentations: Record<string | symbol, Function>;
