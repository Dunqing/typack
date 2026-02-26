import type { CompilerOptions } from './options';
import { type DirectiveTransform, type NodeTransform } from './transform';
import { type CodegenResult } from './codegen';
import type { RootNode } from './ast';
export type TransformPreset = [
    NodeTransform[],
    Record<string, DirectiveTransform>
];
export declare function getBaseTransformPreset(prefixIdentifiers?: boolean): TransformPreset;
export declare function baseCompile(source: string | RootNode, options?: CompilerOptions): CodegenResult;
