import type { PushFn } from '../render';
export declare function ssrRenderSuspense(push: PushFn, { default: renderContent }: Record<string, (() => void) | undefined>): Promise<void>;
