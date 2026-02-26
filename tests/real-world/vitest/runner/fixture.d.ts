import type { Suite, VitestRunner } from './types';
import type { File, FixtureOptions, TestContext } from './types/tasks';
export interface TestFixtureItem extends FixtureOptions {
    name: string;
    value: unknown;
    scope: 'test' | 'file' | 'worker';
    deps: Set<string>;
    parent?: TestFixtureItem;
}
export type UserFixtures = Record<string, unknown>;
export type FixtureRegistrations = Map<string, TestFixtureItem>;
export declare class TestFixtures {
    private _suiteContexts;
    private _overrides;
    private _registrations;
    private static _definitions;
    private static _builtinFixtures;
    private static _fixtureOptionKeys;
    private static _fixtureScopes;
    private static _workerContextSuite;
    static clearDefinitions(): void;
    static getWorkerContexts(): Record<string, any>[];
    static getFileContexts(file: File): Record<string, any>[];
    constructor(registrations?: FixtureRegistrations);
    extend(runner: VitestRunner, userFixtures: UserFixtures): TestFixtures;
    get(suite: Suite): FixtureRegistrations;
    override(runner: VitestRunner, userFixtures: UserFixtures): void;
    getFileContext(file: File): Record<string, any>;
    getWorkerContext(): Record<string, any>;
    private parseUserFixtures;
}
export declare function callFixtureCleanup(context: object): Promise<void>;
/**
 * Returns the current number of cleanup functions registered for the context.
 * This can be used as a checkpoint to later clean up only fixtures added after this point.
 */
export declare function getFixtureCleanupCount(context: object): number;
/**
 * Cleans up only fixtures that were added after the given checkpoint index.
 * This is used by aroundEach to clean up fixtures created inside runTest()
 * while preserving fixtures that were created for aroundEach itself.
 */
export declare function callFixtureCleanupFrom(context: object, fromIndex: number): Promise<void>;
export interface WithFixturesOptions {
    /**
     * Whether this is a suite-level hook (beforeAll/afterAll/aroundAll).
     * Suite hooks can only access file/worker scoped fixtures and static values.
     */
    suiteHook?: 'beforeAll' | 'afterAll' | 'aroundAll';
    /**
     * The test context to use. If not provided, the hookContext passed to the
     * returned function will be used.
     */
    context?: Record<string, any>;
    /**
     * Error with stack trace captured at hook registration time.
     * Used to provide better error messages with proper stack traces.
     */
    stackTraceError?: Error;
    /**
     * Current fixtures from the context.
     */
    fixtures?: TestFixtures;
}
export declare function withFixtures(fn: Function, options?: WithFixturesOptions): (hookContext?: TestContext) => Promise<any>;
interface FixturePropsOptions {
    index?: number;
    original?: Function;
}
export declare function configureProps(fn: Function, options: FixturePropsOptions): void;
export {};
