import type { SourceMap } from 'magic-string';
import type { Plugin, Rollup } from 'vite';
import type { HoistMocksOptions } from './hoistMocks';
export interface HoistMocksPluginOptions extends Omit<HoistMocksOptions, 'regexpHoistable'> {
    include?: string | RegExp | (string | RegExp)[];
    exclude?: string | RegExp | (string | RegExp)[];
    /**
     * overrides include/exclude options
     */
    filter?: (id: string) => boolean;
}
export declare function hoistMocksPlugin(options?: HoistMocksPluginOptions): Plugin;
export declare function hoistMockAndResolve(code: string, id: string, parse: Rollup.PluginContext['parse'], options?: HoistMocksOptions): HoistMocksResult | undefined;
export interface HoistMocksResult {
    code: string;
    map: SourceMap;
}
