import type { CreateMockInstanceProcedure } from '../automocker';
import type { MockedModule, MockedModuleType } from '../registry';
import type { ModuleMockContext, ModuleMockOptions, TestModuleMocker } from '../types';
import type { ModuleMockerInterceptor } from './interceptor';
import { MockerRegistry } from '../registry';
export declare class ModuleMocker implements TestModuleMocker {
    private interceptor;
    private rpc;
    private createMockInstance;
    private config;
    protected registry: MockerRegistry;
    private queue;
    private mockedIds;
    constructor(interceptor: ModuleMockerInterceptor, rpc: ModuleMockerRPC, createMockInstance: CreateMockInstanceProcedure, config: ModuleMockerConfig);
    prepare(): Promise<void>;
    resolveFactoryModule(id: string): Promise<Record<string | symbol, any>>;
    getFactoryModule(id: string): any;
    invalidate(): Promise<void>;
    importActual<T>(id: string, importer: string): Promise<T>;
    protected getBaseUrl(): string;
    importMock<T>(rawId: string, importer: string): Promise<T>;
    mockObject(object: Record<string | symbol, any>, moduleType?: 'automock' | 'autospy'): Record<string | symbol, any>;
    mockObject(object: Record<string | symbol, any>, mockExports: Record<string | symbol, any> | undefined, moduleType?: 'automock' | 'autospy'): Record<string | symbol, any>;
    getMockContext(): ModuleMockContext;
    queueMock(rawId: string, importer: string, factoryOrOptions?: ModuleMockOptions | (() => any)): void;
    queueUnmock(id: string, importer: string): void;
    wrapDynamicImport<T>(moduleFactory: () => Promise<T>): Promise<T>;
    getMockedModuleById(id: string): MockedModule | undefined;
    reset(): void;
    private resolveMockPath;
}
export interface ResolveIdResult {
    id: string;
    url: string;
    optimized: boolean;
}
export interface ResolveMockResult {
    mockType: MockedModuleType;
    resolvedId: string;
    resolvedUrl: string;
    redirectUrl?: string | null;
    needsInterop?: boolean;
}
export interface ModuleMockerRPC {
    invalidate: (ids: string[]) => Promise<void>;
    resolveId: (id: string, importer: string) => Promise<ResolveIdResult | null>;
    resolveMock: (id: string, importer: string, options: {
        mock: 'spy' | 'factory' | 'auto';
    }) => Promise<ResolveMockResult>;
}
export interface ModuleMockerConfig {
    root: string;
}
