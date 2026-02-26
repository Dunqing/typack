import type { FileSpecification, VitestRunner } from './types/runner';
import type { File } from './types/tasks';
export declare function collectTests(specs: string[] | FileSpecification[], runner: VitestRunner): Promise<File[]>;
