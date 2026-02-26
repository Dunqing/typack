import { type App, type VNode } from 'vue';
import { type SSRContext } from './render';
import type { Readable, Writable } from 'node:stream';
export interface SimpleReadable {
    push(chunk: string | null): void;
    destroy(err: any): void;
}
export declare function renderToSimpleStream<T extends SimpleReadable>(input: App | VNode, context: SSRContext, stream: T): T;
/**
 * @deprecated
 */
export declare function renderToStream(input: App | VNode, context?: SSRContext): Readable;
export declare function renderToNodeStream(input: App | VNode, context?: SSRContext): Readable;
export declare function pipeToNodeWritable(input: App | VNode, context: SSRContext | undefined, writable: Writable): void;
export declare function renderToWebStream(input: App | VNode, context?: SSRContext): ReadableStream;
export declare function pipeToWebWritable(input: App | VNode, context: SSRContext | undefined, writable: WritableStream): void;
