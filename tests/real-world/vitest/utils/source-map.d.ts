import type { OriginalMapping } from '@jridgewell/trace-mapping';
import type { ParsedStack, TestError } from './types';
export interface StackTraceParserOptions {
    ignoreStackEntries?: (RegExp | string)[];
    getSourceMap?: (file: string) => unknown;
    getUrlId?: (id: string) => string;
    frameFilter?: (error: TestError, frame: ParsedStack) => boolean | void;
}
declare const stackIgnorePatterns: (string | RegExp)[];
export { stackIgnorePatterns as defaultStackIgnorePatterns };
export declare function parseSingleFFOrSafariStack(raw: string): ParsedStack | null;
export declare function parseSingleStack(raw: string): ParsedStack | null;
export declare function parseSingleV8Stack(raw: string): ParsedStack | null;
export declare function createStackString(stacks: ParsedStack[]): string;
export declare function parseStacktrace(stack: string, options?: StackTraceParserOptions): ParsedStack[];
export declare function parseErrorStacktrace(e: TestError | Error, options?: StackTraceParserOptions): ParsedStack[];
interface SourceMapLike {
    version: number;
    mappings?: string;
    names?: string[];
    sources?: string[];
    sourcesContent?: string[];
    sourceRoot?: string;
}
interface Needle {
    line: number;
    column: number;
}
export declare class DecodedMap {
    map: SourceMapLike;
    _encoded: string;
    _decoded: undefined | number[][][];
    _decodedMemo: Stats;
    url: string;
    version: number;
    names: string[];
    resolvedSources: string[];
    constructor(map: SourceMapLike, from: string);
}
interface Stats {
    lastKey: number;
    lastNeedle: number;
    lastIndex: number;
}
export declare function getOriginalPosition(map: DecodedMap, needle: Needle): OriginalMapping | null;
