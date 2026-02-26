import type { ParsedStack } from '@vitest/utils';
import type { File, Suite } from '../types/tasks';
/**
 * If any tasks been marked as `only`, mark all other tasks as `skip`.
 */
export declare function interpretTaskModes(file: Suite, namePattern?: string | RegExp, testLocations?: number[] | undefined, testIds?: string[] | undefined, testTagsFilter?: ((testTags: string[]) => boolean) | undefined, onlyMode?: boolean, parentIsOnly?: boolean, allowOnly?: boolean): void;
export declare function someTasksAreOnly(suite: Suite): boolean;
export declare function generateHash(str: string): string;
export declare function calculateSuiteHash(parent: Suite): void;
export declare function createFileTask(filepath: string, root: string, projectName: string | undefined, pool?: string, viteEnvironment?: string): File;
/**
 * Generate a unique ID for a file based on its path and project name
 * @param file File relative to the root of the project to keep ID the same between different machines
 * @param projectName The name of the test project
 */
export declare function generateFileHash(file: string, projectName: string | undefined): string;
export declare function findTestFileStackTrace(testFilePath: string, error: string): ParsedStack | undefined;
