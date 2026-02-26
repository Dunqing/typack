import { type ClassComponent, type ComponentInternalInstance, type ComponentOptions, type ConcreteComponent } from './component';
type HMRComponent = ComponentOptions | ClassComponent;
export declare let isHmrUpdating: boolean;
export declare const hmrDirtyComponents: Map<ConcreteComponent, Set<ComponentInternalInstance>>;
export interface HMRRuntime {
    createRecord: typeof createRecord;
    rerender: typeof rerender;
    reload: typeof reload;
}
export declare function registerHMR(instance: ComponentInternalInstance): void;
export declare function unregisterHMR(instance: ComponentInternalInstance): void;
declare function createRecord(id: string, initialDef: HMRComponent): boolean;
declare function rerender(id: string, newRender?: Function): void;
declare function reload(id: string, newComp: HMRComponent): void;
export {};
