import { type RootNode, type SourceLocation } from './ast';
import type { ParserOptions } from './options';
import { type CompilerCompatOptions } from './compat/compatConfig';
type OptionalOptions = 'decodeEntities' | 'whitespace' | 'isNativeTag' | 'isBuiltInComponent' | 'expressionPlugins' | keyof CompilerCompatOptions;
export type MergedParserOptions = Omit<Required<ParserOptions>, OptionalOptions> & Pick<ParserOptions, OptionalOptions>;
export declare const defaultParserOptions: MergedParserOptions;
export declare function cloneLoc(loc: SourceLocation): SourceLocation;
export declare function baseParse(input: string, options?: ParserOptions): RootNode;
export {};
