import { type BindingMetadata, type CompilerError, type ParserOptions, type RawSourceMap, type RootNode, type SourceLocation } from '@vue/compiler-core';
import type { TemplateCompiler } from './compileTemplate';
import type { ImportBinding } from './compileScript';
import type { LRUCache } from 'lru-cache';
export declare const DEFAULT_FILENAME = "anonymous.vue";
export interface SFCParseOptions {
    filename?: string;
    sourceMap?: boolean;
    sourceRoot?: string;
    pad?: boolean | 'line' | 'space';
    ignoreEmpty?: boolean;
    compiler?: TemplateCompiler;
    templateParseOptions?: ParserOptions;
}
export interface SFCBlock {
    type: string;
    content: string;
    attrs: Record<string, string | true>;
    loc: SourceLocation;
    map?: RawSourceMap;
    lang?: string;
    src?: string;
}
export interface SFCTemplateBlock extends SFCBlock {
    type: 'template';
    ast?: RootNode;
}
export interface SFCScriptBlock extends SFCBlock {
    type: 'script';
    setup?: string | boolean;
    bindings?: BindingMetadata;
    imports?: Record<string, ImportBinding>;
    scriptAst?: import('@babel/types').Statement[];
    scriptSetupAst?: import('@babel/types').Statement[];
    warnings?: string[];
    /**
     * Fully resolved dependency file paths (unix slashes) with imported types
     * used in macros, used for HMR cache busting in @vitejs/plugin-vue and
     * vue-loader.
     */
    deps?: string[];
}
export interface SFCStyleBlock extends SFCBlock {
    type: 'style';
    scoped?: boolean;
    module?: string | boolean;
}
export interface SFCDescriptor {
    filename: string;
    source: string;
    template: SFCTemplateBlock | null;
    script: SFCScriptBlock | null;
    scriptSetup: SFCScriptBlock | null;
    styles: SFCStyleBlock[];
    customBlocks: SFCBlock[];
    cssVars: string[];
    /**
     * whether the SFC uses :slotted() modifier.
     * this is used as a compiler optimization hint.
     */
    slotted: boolean;
    /**
     * compare with an existing descriptor to determine whether HMR should perform
     * a reload vs. re-render.
     *
     * Note: this comparison assumes the prev/next script are already identical,
     * and only checks the special case where <script setup lang="ts"> unused import
     * pruning result changes due to template changes.
     */
    shouldForceReload: (prevImports: Record<string, ImportBinding>) => boolean;
}
export interface SFCParseResult {
    descriptor: SFCDescriptor;
    errors: (CompilerError | SyntaxError)[];
}
export declare const parseCache: Map<string, SFCParseResult> | LRUCache<string, SFCParseResult>;
export declare function parse(source: string, options?: SFCParseOptions): SFCParseResult;
/**
 * Note: this comparison assumes the prev/next script are already identical,
 * and only checks the special case where <script setup lang="ts"> unused import
 * pruning result changes due to template changes.
 */
export declare function hmrShouldReload(prevImports: Record<string, ImportBinding>, next: SFCDescriptor): boolean;
