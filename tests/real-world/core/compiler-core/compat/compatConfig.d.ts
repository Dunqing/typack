import type { SourceLocation } from '../ast';
import type { MergedParserOptions } from '../parser';
import type { TransformContext } from '../transform';
export type CompilerCompatConfig = Partial<Record<CompilerDeprecationTypes, boolean | 'suppress-warning'>> & {
    MODE?: 2 | 3;
};
export interface CompilerCompatOptions {
    compatConfig?: CompilerCompatConfig;
}
export declare enum CompilerDeprecationTypes {
    COMPILER_IS_ON_ELEMENT = "COMPILER_IS_ON_ELEMENT",
    COMPILER_V_BIND_SYNC = "COMPILER_V_BIND_SYNC",
    COMPILER_V_BIND_OBJECT_ORDER = "COMPILER_V_BIND_OBJECT_ORDER",
    COMPILER_V_ON_NATIVE = "COMPILER_V_ON_NATIVE",
    COMPILER_V_IF_V_FOR_PRECEDENCE = "COMPILER_V_IF_V_FOR_PRECEDENCE",
    COMPILER_NATIVE_TEMPLATE = "COMPILER_NATIVE_TEMPLATE",
    COMPILER_INLINE_TEMPLATE = "COMPILER_INLINE_TEMPLATE",
    COMPILER_FILTERS = "COMPILER_FILTERS"
}
export declare function isCompatEnabled(key: CompilerDeprecationTypes, context: MergedParserOptions | TransformContext): boolean;
export declare function checkCompatEnabled(key: CompilerDeprecationTypes, context: MergedParserOptions | TransformContext, loc: SourceLocation | null, ...args: any[]): boolean;
export declare function warnDeprecation(key: CompilerDeprecationTypes, context: MergedParserOptions | TransformContext, loc: SourceLocation | null, ...args: any[]): void;
