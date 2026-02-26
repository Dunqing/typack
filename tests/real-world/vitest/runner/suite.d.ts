import type { VitestRunner } from './types/runner';
import type { File, SuiteAPI, SuiteCollector, SuiteHooks, TestAPI } from './types/tasks';
/**
 * Creates a suite of tests, allowing for grouping and hierarchical organization of tests.
 * Suites can contain both tests and other suites, enabling complex test structures.
 *
 * @param {string} name - The name of the suite, used for identification and reporting.
 * @param {Function} fn - A function that defines the tests and suites within this suite.
 * @example
 * ```ts
 * // Define a suite with two tests
 * suite('Math operations', () => {
 *   test('should add two numbers', () => {
 *     expect(add(1, 2)).toBe(3);
 *   });
 *
 *   test('should subtract two numbers', () => {
 *     expect(subtract(5, 2)).toBe(3);
 *   });
 * });
 * ```
 * @example
 * ```ts
 * // Define nested suites
 * suite('String operations', () => {
 *   suite('Trimming', () => {
 *     test('should trim whitespace from start and end', () => {
 *       expect('  hello  '.trim()).toBe('hello');
 *     });
 *   });
 *
 *   suite('Concatenation', () => {
 *     test('should concatenate two strings', () => {
 *       expect('hello' + ' ' + 'world').toBe('hello world');
 *     });
 *   });
 * });
 * ```
 */
export declare const suite: SuiteAPI;
/**
 * Defines a test case with a given name and test function. The test function can optionally be configured with test options.
 *
 * @param {string | Function} name - The name of the test or a function that will be used as a test name.
 * @param {TestOptions | TestFunction} [optionsOrFn] - Optional. The test options or the test function if no explicit name is provided.
 * @param {number | TestOptions | TestFunction} [optionsOrTest] - Optional. The test function or options, depending on the previous parameters.
 * @throws {Error} If called inside another test function.
 * @example
 * ```ts
 * // Define a simple test
 * test('should add two numbers', () => {
 *   expect(add(1, 2)).toBe(3);
 * });
 * ```
 * @example
 * ```ts
 * // Define a test with options
 * test('should subtract two numbers', { retry: 3 }, () => {
 *   expect(subtract(5, 2)).toBe(3);
 * });
 * ```
 */
export declare const test: TestAPI;
/**
 * Creates a suite of tests, allowing for grouping and hierarchical organization of tests.
 * Suites can contain both tests and other suites, enabling complex test structures.
 *
 * @param {string} name - The name of the suite, used for identification and reporting.
 * @param {Function} fn - A function that defines the tests and suites within this suite.
 * @example
 * ```ts
 * // Define a suite with two tests
 * describe('Math operations', () => {
 *   test('should add two numbers', () => {
 *     expect(add(1, 2)).toBe(3);
 *   });
 *
 *   test('should subtract two numbers', () => {
 *     expect(subtract(5, 2)).toBe(3);
 *   });
 * });
 * ```
 * @example
 * ```ts
 * // Define nested suites
 * describe('String operations', () => {
 *   describe('Trimming', () => {
 *     test('should trim whitespace from start and end', () => {
 *       expect('  hello  '.trim()).toBe('hello');
 *     });
 *   });
 *
 *   describe('Concatenation', () => {
 *     test('should concatenate two strings', () => {
 *       expect('hello' + ' ' + 'world').toBe('hello world');
 *     });
 *   });
 * });
 * ```
 */
export declare const describe: SuiteAPI;
/**
 * Defines a test case with a given name and test function. The test function can optionally be configured with test options.
 *
 * @param {string | Function} name - The name of the test or a function that will be used as a test name.
 * @param {TestOptions | TestFunction} [optionsOrFn] - Optional. The test options or the test function if no explicit name is provided.
 * @param {number | TestOptions | TestFunction} [optionsOrTest] - Optional. The test function or options, depending on the previous parameters.
 * @throws {Error} If called inside another test function.
 * @example
 * ```ts
 * // Define a simple test
 * it('adds two numbers', () => {
 *   expect(add(1, 2)).toBe(3);
 * });
 * ```
 * @example
 * ```ts
 * // Define a test with options
 * it('subtracts two numbers', { retry: 3 }, () => {
 *   expect(subtract(5, 2)).toBe(3);
 * });
 * ```
 */
export declare const it: TestAPI;
export declare function getDefaultSuite(): SuiteCollector<object>;
export declare function getTestFilepath(): string;
export declare function getRunner(): VitestRunner;
export declare function clearCollectorContext(file: File, currentRunner: VitestRunner): void;
export declare function getCurrentSuite<ExtraContext = object>(): SuiteCollector<ExtraContext>;
export declare function createSuiteHooks(): SuiteHooks;
export declare function createTaskCollector(fn: (...args: any[]) => any): TestAPI;
