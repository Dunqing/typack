import type { Arrayable, Nullable } from './types';
interface CloneOptions {
    forceWritable?: boolean;
}
interface ErrorOptions {
    message?: string;
    stackTraceLimit?: number;
}
export { nanoid } from './nanoid';
export { shuffle } from './random';
/**
 * Get original stacktrace without source map support the most performant way.
 * - Create only 1 stack frame.
 * - Rewrite prepareStackTrace to bypass "support-stack-trace" (usually takes ~250ms).
 */
export declare function createSimpleStackTrace(options?: ErrorOptions): string;
export declare function notNullish<T>(v: T | null | undefined): v is NonNullable<T>;
export declare function assertTypes(value: unknown, name: string, types: string[]): void;
export declare function isPrimitive(value: unknown): boolean;
export declare function slash(path: string): string;
export declare function cleanUrl(url: string): string;
export declare const isExternalUrl: (url: string) => boolean;
/**
 * Prepend `/@id/` and replace null byte so the id is URL-safe.
 * This is prepended to resolved ids that are not valid browser
 * import specifiers by the importAnalysis plugin.
 */
export declare function wrapId(id: string): string;
/**
 * Undo {@link wrapId}'s `/@id/` and null byte replacements.
 */
export declare function unwrapId(id: string): string;
export declare function withTrailingSlash(path: string): string;
export declare function filterOutComments(s: string): string;
export declare function isBareImport(id: string): boolean;
export declare function toArray<T>(array?: Nullable<Arrayable<T>>): Array<T>;
export declare function isObject(item: unknown): boolean;
export declare function getType(value: unknown): string;
export declare function getOwnProperties(obj: any): (string | symbol)[];
export declare function deepClone<T>(val: T, options?: CloneOptions): T;
export declare function clone<T>(val: T, seen: WeakMap<any, any>, options?: CloneOptions): T;
export declare function noop(): void;
export declare function objectAttr(source: any, path: string, defaultValue?: undefined): any;
export type DeferPromise<T> = Promise<T> & {
    resolve: (value: T | PromiseLike<T>) => void;
    reject: (reason?: any) => void;
};
export declare function createDefer<T>(): DeferPromise<T>;
/**
 * If code starts with a function call, will return its last index, respecting arguments.
 * This will return 25 - last ending character of toMatch ")"
 * Also works with callbacks
 * ```
 * toMatch({ test: '123' });
 * toBeAliased('123')
 * ```
 */
export declare function getCallLastIndex(code: string): number | null;
export declare function isNegativeNaN(val: number): boolean;
/**
 * Deep merge :P
 *
 * Will merge objects only if they are plain
 *
 * Do not merge types - it is very expensive and usually it's better to case a type here
 */
export declare function deepMerge<T extends object = object>(target: T, ...sources: any[]): T;
export declare function unique<T>(array: T[]): T[];
