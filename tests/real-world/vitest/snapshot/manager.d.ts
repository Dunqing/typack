import type { SnapshotResult, SnapshotStateOptions, SnapshotSummary } from './types';
export declare class SnapshotManager {
    options: Omit<SnapshotStateOptions, 'snapshotEnvironment'>;
    summary: SnapshotSummary;
    extension: string;
    constructor(options: Omit<SnapshotStateOptions, 'snapshotEnvironment'>);
    clear(): void;
    add(result: SnapshotResult): void;
    resolvePath<T = any>(testPath: string, context?: T): string;
    resolveRawPath(testPath: string, rawPath: string): string;
}
export declare function emptySummary(options: Omit<SnapshotStateOptions, 'snapshotEnvironment'>): SnapshotSummary;
export declare function addSnapshotResult(summary: SnapshotSummary, result: SnapshotResult): void;
