import type { Suite, Task } from '../types/tasks';
/**
 * Partition in tasks groups by consecutive concurrent
 */
export declare function partitionSuiteChildren(suite: Suite): Task[][];
