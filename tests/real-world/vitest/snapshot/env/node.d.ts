import type { SnapshotEnvironment, SnapshotEnvironmentOptions } from '../types';
export declare class NodeSnapshotEnvironment implements SnapshotEnvironment {
    private options;
    constructor(options?: SnapshotEnvironmentOptions);
    getVersion(): string;
    getHeader(): string;
    resolveRawPath(testPath: string, rawPath: string): Promise<string>;
    resolvePath(filepath: string): Promise<string>;
    prepareDirectory(dirPath: string): Promise<void>;
    saveSnapshotFile(filepath: string, snapshot: string): Promise<void>;
    readSnapshotFile(filepath: string): Promise<string | null>;
    removeSnapshotFile(filepath: string): Promise<void>;
}
