import type { RawSourceMap } from '@vue/compiler-core';
import type { SFCStyleCompileOptions } from '../compileStyle';
export type StylePreprocessor = (source: string, map: RawSourceMap | undefined, options: {
    [key: string]: any;
    additionalData?: string | ((source: string, filename: string) => string);
    filename: string;
}, customRequire: SFCStyleCompileOptions['preprocessCustomRequire']) => StylePreprocessorResults;
export interface StylePreprocessorResults {
    code: string;
    map?: object;
    errors: Error[];
    dependencies: string[];
}
export type PreprocessLang = 'less' | 'sass' | 'scss' | 'styl' | 'stylus';
export declare const processors: Record<PreprocessLang, StylePreprocessor>;
