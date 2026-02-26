import type { Plugin } from 'vite';
import { MockerRegistry } from '../registry';
export interface InterceptorPluginOptions {
    /**
     * @default "__vitest_mocker__"
     */
    globalThisAccessor?: string;
    registry?: MockerRegistry;
}
export declare function interceptorPlugin(options?: InterceptorPluginOptions): Plugin;
