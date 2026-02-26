import type { SetupWorker, StartOptions } from 'msw/browser';
import type { MockedModule } from '../registry';
import type { ModuleMockerInterceptor } from './interceptor';
import { MockerRegistry } from '../registry';
export interface ModuleMockerMSWInterceptorOptions {
    /**
     * The identifier to access the globalThis object in the worker.
     * This will be injected into the script as is, so make sure it's a valid JS expression.
     * @example
     * ```js
     * // globalThisAccessor: '__my_variable__' produces:
     * globalThis[__my_variable__]
     * // globalThisAccessor: 'Symbol.for('secret:mocks')' produces:
     * globalThis[Symbol.for('secret:mocks')]
     * // globalThisAccessor: '"__vitest_mocker__"' (notice quotes) produces:
     * globalThis["__vitest_mocker__"]
     * ```
     * @default `"__vitest_mocker__"`
     */
    globalThisAccessor?: string;
    /**
     * Options passed down to `msw.setupWorker().start(options)`
     */
    mswOptions?: StartOptions;
    /**
     * A pre-configured `msw.setupWorker` instance.
     */
    mswWorker?: SetupWorker;
}
export declare class ModuleMockerMSWInterceptor implements ModuleMockerInterceptor {
    private readonly options;
    protected readonly mocks: MockerRegistry;
    private startPromise;
    private worker;
    constructor(options?: ModuleMockerMSWInterceptorOptions);
    register(module: MockedModule): Promise<void>;
    delete(url: string): Promise<void>;
    invalidate(): Promise<void>;
    private resolveManualMock;
    protected init(): Promise<SetupWorker>;
}
