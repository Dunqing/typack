import { type App, type VNode } from 'vue';
import { type SSRBuffer, type SSRContext } from './render';
export declare function unrollBuffer(buffer: SSRBuffer): Promise<string> | string;
export declare function renderToString(input: App | VNode, context?: SSRContext): Promise<string>;
export declare function resolveTeleports(context: SSRContext): Promise<void>;
