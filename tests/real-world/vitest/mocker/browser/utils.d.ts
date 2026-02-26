import type { ViteHotContext } from 'vite/types/hot.js';
declare const hot: ViteHotContext;
export { hot };
export declare function rpc<T>(event: string, data?: any): Promise<T>;
