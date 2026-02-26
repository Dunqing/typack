/**
 * Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { ParsedStack } from '@vitest/utils';
import type { SnapshotEnvironment, SnapshotMatchOptions, SnapshotResult, SnapshotStateOptions } from '../types';
import { CounterMap } from './utils';
interface SnapshotReturnOptions {
    actual: string;
    count: number;
    expected?: string;
    key: string;
    pass: boolean;
}
interface SaveStatus {
    deleted: boolean;
    saved: boolean;
}
export default class SnapshotState {
    testFilePath: string;
    snapshotPath: string;
    private _counters;
    private _dirty;
    private _updateSnapshot;
    private _snapshotData;
    private _initialData;
    private _inlineSnapshots;
    private _inlineSnapshotStacks;
    private _testIdToKeys;
    private _rawSnapshots;
    private _uncheckedKeys;
    private _snapshotFormat;
    private _environment;
    private _fileExists;
    expand: boolean;
    private _added;
    private _matched;
    private _unmatched;
    private _updated;
    get added(): CounterMap<string>;
    set added(value: number);
    get matched(): CounterMap<string>;
    set matched(value: number);
    get unmatched(): CounterMap<string>;
    set unmatched(value: number);
    get updated(): CounterMap<string>;
    set updated(value: number);
    private constructor();
    static create(testFilePath: string, options: SnapshotStateOptions): Promise<SnapshotState>;
    get environment(): SnapshotEnvironment;
    markSnapshotsAsCheckedForTest(testName: string): void;
    clearTest(testId: string): void;
    protected _inferInlineSnapshotStack(stacks: ParsedStack[]): ParsedStack | null;
    private _addSnapshot;
    save(): Promise<SaveStatus>;
    getUncheckedCount(): number;
    getUncheckedKeys(): Array<string>;
    removeUncheckedKeys(): void;
    match({ testId, testName, received, key, inlineSnapshot, isInline, error, rawSnapshot, }: SnapshotMatchOptions): SnapshotReturnOptions;
    pack(): Promise<SnapshotResult>;
}
export {};
