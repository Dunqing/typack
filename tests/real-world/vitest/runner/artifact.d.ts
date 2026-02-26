import type { Test, TestArtifact, TestAttachment } from './types/tasks';
/**
 * @experimental
 * @advanced
 *
 * Records a custom test artifact during test execution.
 *
 * This function allows you to attach structured data, files, or metadata to a test.
 *
 * Vitest automatically injects the source location where the artifact was created and manages any attachments you include.
 *
 * **Note:** artifacts must be recorded before the task is reported. Any artifacts recorded after that will not be included in the task.
 *
 * @param task - The test task context, typically accessed via `this.task` in custom matchers or `context.task` in tests
 * @param artifact - The artifact to record. Must extend {@linkcode TestArtifactBase}
 *
 * @returns A promise that resolves to the recorded artifact with location injected
 *
 * @throws {Error} If the test runner doesn't support artifacts
 *
 * @example
 * ```ts
 * // In a custom assertion
 * async function toHaveValidSchema(this: MatcherState, actual: unknown) {
 *   const validation = validateSchema(actual)
 *
 *   await recordArtifact(this.task, {
 *     type: 'my-plugin:schema-validation',
 *     passed: validation.valid,
 *     errors: validation.errors,
 *   })
 *
 *   return { pass: validation.valid, message: () => '...' }
 * }
 * ```
 */
export declare function recordArtifact<Artifact extends TestArtifact>(task: Test, artifact: Artifact): Promise<Artifact>;
/**
 * Records an async operation associated with a test task.
 *
 * This function tracks promises that should be awaited before a test completes.
 * The promise is automatically removed from the test's promise list once it settles.
 */
export declare function recordAsyncOperation<T>(test: Test, promise: Promise<T>): Promise<T>;
/**
 * Validates and prepares a test attachment for serialization.
 *
 * This function ensures attachments have either `body` or `path` set (but not both), and converts `Uint8Array` bodies to base64-encoded strings for easier serialization.
 *
 * @param attachment - The attachment to validate and prepare
 *
 * @throws {TypeError} If neither `body` nor `path` is provided
 * @throws {TypeError} If both `body` and `path` are provided
 */
export declare function manageArtifactAttachment(attachment: TestAttachment): void;
