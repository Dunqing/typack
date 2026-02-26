import type { PrettyFormatOptions } from '@vitest/pretty-format';
type Inspect = (value: unknown, options: Options) => string;
interface Options {
    showHidden: boolean;
    depth: number;
    colors: boolean;
    customInspect: boolean;
    showProxy: boolean;
    maxArrayLength: number;
    breakLength: number;
    truncate: number;
    seen: unknown[];
    inspect: Inspect;
    stylize: (value: string, styleType: string) => string;
}
export type LoupeOptions = Partial<Options>;
export interface StringifyOptions extends PrettyFormatOptions {
    maxLength?: number;
    filterNode?: string | ((node: any) => boolean);
}
export declare function stringify(object: unknown, maxDepth?: number, { maxLength, filterNode, ...options }?: StringifyOptions): string;
export declare const formatRegExp: RegExp;
export declare function format(...args: unknown[]): string;
export declare function browserFormat(...args: unknown[]): string;
export declare function inspect(obj: unknown, options?: LoupeOptions): string;
export declare function objDisplay(obj: unknown, options?: LoupeOptions): string;
export {};
