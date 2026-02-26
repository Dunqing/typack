import type { TestTagDefinition, VitestRunnerConfig } from '../types/runner';
export declare function validateTags(config: VitestRunnerConfig, tags: string[]): void;
export declare function createNoTagsError(availableTags: TestTagDefinition[], tag: string, prefix?: string): never;
export declare function createTagsFilter(tagsExpr: string[], availableTags: TestTagDefinition[]): (testTags: string[]) => boolean;
