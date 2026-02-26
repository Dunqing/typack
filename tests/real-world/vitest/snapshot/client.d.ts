import type { RawSnapshotInfo } from './port/rawSnapshot';
import type { SnapshotResult, SnapshotStateOptions } from './types';
import SnapshotState from './port/state';
export interface Context {
    file: string;
    title?: string;
    fullTitle?: string;
}
interface AssertOptions {
    received: unknown;
    filepath: string;
    name: string;
    /**
     * Not required but needed for `SnapshotClient.clearTest` to implement test-retry behavior.
     * @default name
     */
    testId?: string;
    message?: string;
    isInline?: boolean;
    properties?: object;
    inlineSnapshot?: string;
    error?: Error;
    errorMessage?: string;
    rawSnapshot?: RawSnapshotInfo;
}
export interface SnapshotClientOptions {
    isEqual?: (received: unknown, expected: unknown) => boolean;
}
export declare class SnapshotClient {
    private options;
    snapshotStateMap: Map<string, SnapshotState>;
    constructor(options?: SnapshotClientOptions);
    setup(filepath: string, options: SnapshotStateOptions): Promise<void>;
    finish(filepath: string): Promise<SnapshotResult>;
    skipTest(filepath: string, testName: string): void;
    clearTest(filepath: string, testId: string): void;
    getSnapshotState(filepath: string): SnapshotState;
    assert(options: AssertOptions): void;
    assertRaw(options: AssertOptions): Promise<void>;
    clear(): void;
}
export {};
