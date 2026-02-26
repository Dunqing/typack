export declare const version: string;
export { parse } from './compiler-sfc/parse';
export { compileTemplate } from './compiler-sfc/compileTemplate';
export { compileStyle, compileStyleAsync } from './compiler-sfc/compileStyle';
export { compileScript } from './compiler-sfc/compileScript';
export { rewriteDefault, rewriteDefaultAST } from './compiler-sfc/rewriteDefault';
export { resolveTypeElements, inferRuntimeType } from './compiler-sfc/script/resolveType';
import { type SFCParseResult } from './compiler-sfc/parse';
export declare const parseCache: Map<string, SFCParseResult>;
export declare const errorMessages: Record<number, string>;
export { parse as babelParse } from '@babel/parser';
import MagicString from 'magic-string';
export { MagicString };
export declare const walk: any;
export { generateCodeFrame, walkIdentifiers, extractIdentifiers, isInDestructureAssignment, isStaticProperty, } from '@vue/compiler-core';
export { invalidateTypeCache, registerTS } from './compiler-sfc/script/resolveType';
export { extractRuntimeProps } from './compiler-sfc/script/defineProps';
export { extractRuntimeEmits } from './compiler-sfc/script/defineEmits';
export type { SFCParseOptions, SFCParseResult, SFCDescriptor, SFCBlock, SFCTemplateBlock, SFCScriptBlock, SFCStyleBlock, } from './compiler-sfc/parse';
export type { TemplateCompiler, SFCTemplateCompileOptions, SFCTemplateCompileResults, } from './compiler-sfc/compileTemplate';
export type { SFCStyleCompileOptions, SFCAsyncStyleCompileOptions, SFCStyleCompileResults, } from './compiler-sfc/compileStyle';
export type { SFCScriptCompileOptions } from './compiler-sfc/compileScript';
export type { ScriptCompileContext } from './compiler-sfc/script/context';
export type { TypeResolveContext, SimpleTypeResolveOptions, SimpleTypeResolveContext, } from './compiler-sfc/script/resolveType';
export type { AssetURLOptions, AssetURLTagConfig, } from './compiler-sfc/template/transformAssetUrl';
export type { CompilerOptions, CompilerError, BindingMetadata, } from '@vue/compiler-core';
/**
 * @deprecated this is preserved to avoid breaking vite-plugin-vue < 5.0
 * with reactivityTransform: true. The desired behavior should be silently
 * ignoring the option instead of breaking.
 */
export declare const shouldTransformRef: () => boolean;
