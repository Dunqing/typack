import type { MaybeMockedDeep } from '@vitest/spy';
import type { ModuleMockFactoryWithHelper, ModuleMockOptions } from '../types';
export interface CompilerHintsOptions {
    /**
     * This is the key used to access the globalThis object in the worker.
     * Unlike `globalThisAccessor` in other APIs, this is not injected into the script.
     * ```ts
     * // globalThisKey: '__my_variable__' produces:
     * globalThis['__my_variable__']
     * // globalThisKey: '"__my_variable__"' produces:
     * globalThis['"__my_variable__"'] // notice double quotes
     * ```
     * @default '__vitest_mocker__'
     */
    globalThisKey?: string;
}
export interface ModuleMockerCompilerHints {
    hoisted: <T>(factory: () => T) => T;
    mock: (path: string | Promise<unknown>, factory?: ModuleMockOptions | ModuleMockFactoryWithHelper) => void;
    unmock: (path: string | Promise<unknown>) => void;
    doMock: (path: string | Promise<unknown>, factory?: ModuleMockOptions | ModuleMockFactoryWithHelper) => void;
    doUnmock: (path: string | Promise<unknown>) => void;
    importActual: <T>(path: string) => Promise<T>;
    importMock: <T>(path: string) => Promise<MaybeMockedDeep<T>>;
}
export declare function createCompilerHints(options?: CompilerHintsOptions): ModuleMockerCompilerHints;
