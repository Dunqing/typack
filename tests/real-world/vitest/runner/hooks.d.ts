import type { AfterAllListener, AfterEachListener, AroundAllListener, AroundEachListener, BeforeAllListener, BeforeEachListener, OnTestFailedHandler, OnTestFinishedHandler, TaskHook, TestContext } from './types/tasks';
export declare function getBeforeHookCleanupCallback(hook: Function, result: any, context?: TestContext): Function | undefined;
/**
 * Registers a callback function to be executed once before all tests within the current suite.
 * This hook is useful for scenarios where you need to perform setup operations that are common to all tests in a suite, such as initializing a database connection or setting up a test environment.
 *
 * **Note:** The `beforeAll` hooks are executed in the order they are defined one after another. You can configure this by changing the `sequence.hooks` option in the config file.
 *
 * @param {Function} fn - The callback function to be executed before all tests.
 * @param {number} [timeout] - Optional timeout in milliseconds for the hook. If not provided, the default hook timeout from the runner's configuration is used.
 * @returns {void}
 * @example
 * ```ts
 * // Example of using beforeAll to set up a database connection
 * beforeAll(async () => {
 *   await database.connect();
 * });
 * ```
 */
export declare function beforeAll<ExtraContext = object>(this: unknown, fn: BeforeAllListener<ExtraContext>, timeout?: number): void;
/**
 * Registers a callback function to be executed once after all tests within the current suite have completed.
 * This hook is useful for scenarios where you need to perform cleanup operations after all tests in a suite have run, such as closing database connections or cleaning up temporary files.
 *
 * **Note:** The `afterAll` hooks are running in reverse order of their registration. You can configure this by changing the `sequence.hooks` option in the config file.
 *
 * @param {Function} fn - The callback function to be executed after all tests.
 * @param {number} [timeout] - Optional timeout in milliseconds for the hook. If not provided, the default hook timeout from the runner's configuration is used.
 * @returns {void}
 * @example
 * ```ts
 * // Example of using afterAll to close a database connection
 * afterAll(async () => {
 *   await database.disconnect();
 * });
 * ```
 */
export declare function afterAll<ExtraContext = object>(this: unknown, fn: AfterAllListener<ExtraContext>, timeout?: number): void;
/**
 * Registers a callback function to be executed before each test within the current suite.
 * This hook is useful for scenarios where you need to reset or reinitialize the test environment before each test runs, such as resetting database states, clearing caches, or reinitializing variables.
 *
 * **Note:** The `beforeEach` hooks are executed in the order they are defined one after another. You can configure this by changing the `sequence.hooks` option in the config file.
 *
 * @param {Function} fn - The callback function to be executed before each test. This function receives an `TestContext` parameter if additional test context is needed.
 * @param {number} [timeout] - Optional timeout in milliseconds for the hook. If not provided, the default hook timeout from the runner's configuration is used.
 * @returns {void}
 * @example
 * ```ts
 * // Example of using beforeEach to reset a database state
 * beforeEach(async () => {
 *   await database.reset();
 * });
 * ```
 */
export declare function beforeEach<ExtraContext = object>(fn: BeforeEachListener<ExtraContext>, timeout?: number): void;
/**
 * Registers a callback function to be executed after each test within the current suite has completed.
 * This hook is useful for scenarios where you need to clean up or reset the test environment after each test runs, such as deleting temporary files, clearing test-specific database entries, or resetting mocked functions.
 *
 * **Note:** The `afterEach` hooks are running in reverse order of their registration. You can configure this by changing the `sequence.hooks` option in the config file.
 *
 * @param {Function} fn - The callback function to be executed after each test. This function receives an `TestContext` parameter if additional test context is needed.
 * @param {number} [timeout] - Optional timeout in milliseconds for the hook. If not provided, the default hook timeout from the runner's configuration is used.
 * @returns {void}
 * @example
 * ```ts
 * // Example of using afterEach to delete temporary files created during a test
 * afterEach(async () => {
 *   await fileSystem.deleteTempFiles();
 * });
 * ```
 */
export declare function afterEach<ExtraContext = object>(fn: AfterEachListener<ExtraContext>, timeout?: number): void;
/**
 * Registers a callback function to be executed when a test fails within the current suite.
 * This function allows for custom actions to be performed in response to test failures, such as logging, cleanup, or additional diagnostics.
 *
 * **Note:** The `onTestFailed` hooks are running in reverse order of their registration. You can configure this by changing the `sequence.hooks` option in the config file.
 *
 * @param {Function} fn - The callback function to be executed upon a test failure. The function receives the test result (including errors).
 * @param {number} [timeout] - Optional timeout in milliseconds for the hook. If not provided, the default hook timeout from the runner's configuration is used.
 * @throws {Error} Throws an error if the function is not called within a test.
 * @returns {void}
 * @example
 * ```ts
 * // Example of using onTestFailed to log failure details
 * onTestFailed(({ errors }) => {
 *   console.log(`Test failed: ${test.name}`, errors);
 * });
 * ```
 */
export declare const onTestFailed: TaskHook<OnTestFailedHandler>;
/**
 * Registers a callback function to be executed when the current test finishes, regardless of the outcome (pass or fail).
 * This function is ideal for performing actions that should occur after every test execution, such as cleanup, logging, or resetting shared resources.
 *
 * This hook is useful if you have access to a resource in the test itself and you want to clean it up after the test finishes. It is a more compact way to clean up resources than using the combination of `beforeEach` and `afterEach`.
 *
 * **Note:** The `onTestFinished` hooks are running in reverse order of their registration. You can configure this by changing the `sequence.hooks` option in the config file.
 *
 * **Note:** The `onTestFinished` hook is not called if the test is canceled with a dynamic `ctx.skip()` call.
 *
 * @param {Function} fn - The callback function to be executed after a test finishes. The function can receive parameters providing details about the completed test, including its success or failure status.
 * @param {number} [timeout] - Optional timeout in milliseconds for the hook. If not provided, the default hook timeout from the runner's configuration is used.
 * @throws {Error} Throws an error if the function is not called within a test.
 * @returns {void}
 * @example
 * ```ts
 * // Example of using onTestFinished for cleanup
 * const db = await connectToDatabase();
 * onTestFinished(async () => {
 *   await db.disconnect();
 * });
 * ```
 */
export declare const onTestFinished: TaskHook<OnTestFinishedHandler>;
/**
 * Registers a callback function that wraps around all tests within the current suite.
 * The callback receives a `runSuite` function that must be called to run the suite's tests.
 * This hook is useful for scenarios where you need to wrap an entire suite in a context
 * (e.g., starting a server, opening a database connection that all tests share).
 *
 * **Note:** When multiple `aroundAll` hooks are registered, they are nested inside each other.
 * The first registered hook is the outermost wrapper.
 *
 * @param {Function} fn - The callback function that wraps the suite. Must call `runSuite()` to run the tests.
 * @param {number} [timeout] - Optional timeout in milliseconds for the hook. If not provided, the default hook timeout from the runner's configuration is used.
 * @returns {void}
 * @example
 * ```ts
 * // Example of using aroundAll to wrap suite in a tracing span
 * aroundAll(async (runSuite) => {
 *   await tracer.trace('test-suite', runSuite);
 * });
 * ```
 * @example
 * ```ts
 * // Example of using aroundAll with fixtures
 * aroundAll(async (runSuite, { db }) => {
 *   await db.transaction(() => runSuite());
 * });
 * ```
 */
export declare function aroundAll<ExtraContext = object>(this: unknown, fn: AroundAllListener<ExtraContext>, timeout?: number): void;
/**
 * Registers a callback function that wraps around each test within the current suite.
 * The callback receives a `runTest` function that must be called to run the test.
 * This hook is useful for scenarios where you need to wrap tests in a context (e.g., database transactions).
 *
 * **Note:** When multiple `aroundEach` hooks are registered, they are nested inside each other.
 * The first registered hook is the outermost wrapper.
 *
 * @param {Function} fn - The callback function that wraps the test. Must call `runTest()` to run the test.
 * @param {number} [timeout] - Optional timeout in milliseconds for the hook. If not provided, the default hook timeout from the runner's configuration is used.
 * @returns {void}
 * @example
 * ```ts
 * // Example of using aroundEach to wrap tests in a database transaction
 * aroundEach(async (runTest) => {
 *   await database.transaction(() => runTest());
 * });
 * ```
 * @example
 * ```ts
 * // Example of using aroundEach with fixtures
 * aroundEach(async (runTest, { db }) => {
 *   await db.transaction(() => runTest());
 * });
 * ```
 */
export declare function aroundEach<ExtraContext = object>(fn: AroundEachListener<ExtraContext>, timeout?: number): void;
export declare function getAroundHookTimeout(hook: Function): number;
export declare function getAroundHookStackTrace(hook: Function): Error | undefined;
