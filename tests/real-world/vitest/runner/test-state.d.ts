import type { Test } from './types/tasks';
export declare function setCurrentTest<T extends Test>(test: T | undefined): void;
export declare function getCurrentTest<T extends Test | undefined>(): T;
export declare function addRunningTest(test: Test): () => void;
export declare function getRunningTests(): Array<Test>;
