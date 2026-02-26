import type { App } from './apiCreateApp';
import type { ComponentInternalInstance } from './component';
interface AppRecord {
    id: number;
    app: App;
    version: string;
    types: Record<string, string | Symbol>;
}
export interface DevtoolsHook {
    enabled?: boolean;
    emit: (event: string, ...payload: any[]) => void;
    on: (event: string, handler: Function) => void;
    once: (event: string, handler: Function) => void;
    off: (event: string, handler: Function) => void;
    appRecords: AppRecord[];
    /**
     * Added at https://github.com/vuejs/devtools/commit/f2ad51eea789006ab66942e5a27c0f0986a257f9
     * Returns whether the arg was buffered or not
     */
    cleanupBuffer?: (matchArg: unknown) => boolean;
}
export declare let devtools: DevtoolsHook;
export declare function setDevtoolsHook(hook: DevtoolsHook, target: any): void;
export declare function devtoolsInitApp(app: App, version: string): void;
export declare function devtoolsUnmountApp(app: App): void;
export declare const devtoolsComponentAdded: DevtoolsComponentHook;
export declare const devtoolsComponentUpdated: DevtoolsComponentHook;
export declare const devtoolsComponentRemoved: (component: ComponentInternalInstance) => void;
type DevtoolsComponentHook = (component: ComponentInternalInstance) => void;
export declare const devtoolsPerfStart: DevtoolsPerformanceHook;
export declare const devtoolsPerfEnd: DevtoolsPerformanceHook;
type DevtoolsPerformanceHook = (component: ComponentInternalInstance, type: string, time: number) => void;
export declare function devtoolsComponentEmit(component: ComponentInternalInstance, event: string, params: any[]): void;
export {};
