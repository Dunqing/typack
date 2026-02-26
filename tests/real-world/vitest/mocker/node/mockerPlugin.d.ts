import type { Plugin } from 'vite';
import type { AutomockPluginOptions } from './automockPlugin';
import type { HoistMocksPluginOptions } from './hoistMocksPlugin';
interface MockerPluginOptions extends AutomockPluginOptions {
    hoistMocks?: HoistMocksPluginOptions;
}
export declare function mockerPlugin(options?: MockerPluginOptions): Plugin[];
export {};
