import { type CodegenResult, type CompilerError, type CompilerOptions, type ParserOptions, type RawSourceMap, type RootNode } from '@vue/compiler-core';
import { type AssetURLOptions, type AssetURLTagConfig } from './template/transformAssetUrl';
export interface TemplateCompiler {
    compile(source: string | RootNode, options: CompilerOptions): CodegenResult;
    parse(template: string, options: ParserOptions): RootNode;
}
export interface SFCTemplateCompileResults {
    code: string;
    ast?: RootNode;
    preamble?: string;
    source: string;
    tips: string[];
    errors: (string | CompilerError)[];
    map?: RawSourceMap;
}
export interface SFCTemplateCompileOptions {
    source: string;
    ast?: RootNode;
    filename: string;
    id: string;
    scoped?: boolean;
    slotted?: boolean;
    isProd?: boolean;
    ssr?: boolean;
    ssrCssVars?: string[];
    inMap?: RawSourceMap;
    compiler?: TemplateCompiler;
    compilerOptions?: CompilerOptions;
    preprocessLang?: string;
    preprocessOptions?: any;
    /**
     * In some cases, compiler-sfc may not be inside the project root (e.g. when
     * linked or globally installed). In such cases a custom `require` can be
     * passed to correctly resolve the preprocessors.
     */
    preprocessCustomRequire?: (id: string) => any;
    /**
     * Configure what tags/attributes to transform into asset url imports,
     * or disable the transform altogether with `false`.
     */
    transformAssetUrls?: AssetURLOptions | AssetURLTagConfig | boolean;
}
export declare function compileTemplate(options: SFCTemplateCompileOptions): SFCTemplateCompileResults;
