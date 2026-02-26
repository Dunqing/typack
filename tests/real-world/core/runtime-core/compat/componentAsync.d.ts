import type { Component } from '../component';
interface LegacyAsyncOptions {
    component: Promise<Component>;
    loading?: Component;
    error?: Component;
    delay?: number;
    timeout?: number;
}
type LegacyAsyncReturnValue = Promise<Component> | LegacyAsyncOptions;
type LegacyAsyncComponent = (resolve?: (res: LegacyAsyncReturnValue) => void, reject?: (reason?: any) => void) => LegacyAsyncReturnValue | undefined;
export declare function convertLegacyAsyncComponent(comp: LegacyAsyncComponent): Component;
export {};
