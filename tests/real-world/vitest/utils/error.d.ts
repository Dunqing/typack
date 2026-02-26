import type { DiffOptions } from './diff';
import type { TestError } from './types';
import { serializeValue } from './serialize';
export { serializeValue as serializeError };
export declare function processError(_err: any, diffOptions?: DiffOptions, seen?: WeakSet<WeakKey>): TestError;
