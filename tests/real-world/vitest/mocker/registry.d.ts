export declare class MockerRegistry {
    private readonly registryByUrl;
    private readonly registryById;
    clear(): void;
    keys(): IterableIterator<string>;
    add(mock: MockedModule): void;
    register(json: MockedModuleSerialized): MockedModule;
    register(type: 'redirect', raw: string, id: string, url: string, redirect: string): RedirectedModule;
    register(type: 'manual', raw: string, id: string, url: string, factory: () => any): ManualMockedModule;
    register(type: 'automock', raw: string, id: string, url: string): AutomockedModule;
    register(type: 'autospy', id: string, raw: string, url: string): AutospiedModule;
    delete(id: string): void;
    deleteById(id: string): void;
    get(id: string): MockedModule | undefined;
    getById(id: string): MockedModule | undefined;
    has(id: string): boolean;
}
export type MockedModule = AutomockedModule | AutospiedModule | ManualMockedModule | RedirectedModule;
export type MockedModuleType = 'automock' | 'autospy' | 'manual' | 'redirect';
export type MockedModuleSerialized = AutomockedModuleSerialized | AutospiedModuleSerialized | ManualMockedModuleSerialized | RedirectedModuleSerialized;
export declare class AutomockedModule {
    raw: string;
    id: string;
    url: string;
    readonly type = "automock";
    constructor(raw: string, id: string, url: string);
    static fromJSON(data: AutomockedModuleSerialized): AutospiedModule;
    toJSON(): AutomockedModuleSerialized;
}
export interface AutomockedModuleSerialized {
    type: 'automock';
    url: string;
    raw: string;
    id: string;
}
export declare class AutospiedModule {
    raw: string;
    id: string;
    url: string;
    readonly type = "autospy";
    constructor(raw: string, id: string, url: string);
    static fromJSON(data: AutospiedModuleSerialized): AutospiedModule;
    toJSON(): AutospiedModuleSerialized;
}
export interface AutospiedModuleSerialized {
    type: 'autospy';
    url: string;
    raw: string;
    id: string;
}
export declare class RedirectedModule {
    raw: string;
    id: string;
    url: string;
    redirect: string;
    readonly type = "redirect";
    constructor(raw: string, id: string, url: string, redirect: string);
    static fromJSON(data: RedirectedModuleSerialized): RedirectedModule;
    toJSON(): RedirectedModuleSerialized;
}
export interface RedirectedModuleSerialized {
    type: 'redirect';
    url: string;
    id: string;
    raw: string;
    redirect: string;
}
export declare class ManualMockedModule<T = any> {
    raw: string;
    id: string;
    url: string;
    factory: () => T;
    cache: T | undefined;
    readonly type = "manual";
    constructor(raw: string, id: string, url: string, factory: () => T);
    resolve(): T;
    static fromJSON(data: ManualMockedModuleSerialized, factory: () => any): ManualMockedModule;
    toJSON(): ManualMockedModuleSerialized;
}
export interface ManualMockedModuleSerialized {
    type: 'manual';
    url: string;
    id: string;
    raw: string;
}
