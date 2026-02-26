import { type LazyResult, type Result } from 'postcss';
import { type PreprocessLang } from './style/preprocessors';
import type { RawSourceMap } from '@vue/compiler-core';
export interface SFCStyleCompileOptions {
    source: string;
    filename: string;
    id: string;
    scoped?: boolean;
    trim?: boolean;
    isProd?: boolean;
    inMap?: RawSourceMap;
    preprocessLang?: PreprocessLang;
    preprocessOptions?: any;
    preprocessCustomRequire?: (id: string) => any;
    postcssOptions?: any;
    postcssPlugins?: any[];
    /**
     * @deprecated use `inMap` instead.
     */
    map?: RawSourceMap;
}
/**
 * Aligns with postcss-modules
 * https://github.com/css-modules/postcss-modules
 */
export interface CSSModulesOptions {
    scopeBehaviour?: 'global' | 'local';
    generateScopedName?: string | ((name: string, filename: string, css: string) => string);
    hashPrefix?: string;
    localsConvention?: 'camelCase' | 'camelCaseOnly' | 'dashes' | 'dashesOnly';
    exportGlobals?: boolean;
    globalModulePaths?: RegExp[];
}
export interface SFCAsyncStyleCompileOptions extends SFCStyleCompileOptions {
    isAsync?: boolean;
    modules?: boolean;
    modulesOptions?: CSSModulesOptions;
}
export interface SFCStyleCompileResults {
    code: string;
    map: RawSourceMap | undefined;
    rawResult: Result | LazyResult | undefined;
    errors: Error[];
    modules?: Record<string, string>;
    dependencies: Set<string>;
}
export declare function compileStyle(options: SFCStyleCompileOptions): SFCStyleCompileResults;
export declare function compileStyleAsync(options: SFCAsyncStyleCompileOptions): Promise<SFCStyleCompileResults>;
export declare function doCompileStyle(options: SFCAsyncStyleCompileOptions): SFCStyleCompileResults | Promise<SFCStyleCompileResults>;
