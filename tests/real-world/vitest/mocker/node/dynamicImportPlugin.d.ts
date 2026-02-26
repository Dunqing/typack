import type { SourceMap } from 'magic-string';
import type { Plugin, Rollup } from 'vite';
export interface DynamicImportPluginOptions {
    /**
     * @default `"__vitest_mocker__"`
     */
    globalThisAccessor?: string;
    filter?: (id: string) => boolean;
}
export declare function dynamicImportPlugin(options?: DynamicImportPluginOptions): Plugin;
export interface DynamicImportInjectorResult {
    code: string;
    map: SourceMap;
}
export declare function injectDynamicImport(code: string, id: string, parse: Rollup.PluginContext['parse'], options?: DynamicImportPluginOptions): DynamicImportInjectorResult | undefined;
