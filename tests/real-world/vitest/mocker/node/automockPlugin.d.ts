import type { Plugin } from 'vite';
import type { AutomockOptions } from './automock';
export type { AutomockOptions as AutomockPluginOptions } from './automock';
export declare function automockPlugin(options?: AutomockOptions): Plugin;
