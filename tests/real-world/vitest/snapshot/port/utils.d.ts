/**
 * Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { OptionsReceived as PrettyFormatOptions } from '@vitest/pretty-format';
import type { SnapshotData, SnapshotStateOptions } from '../types';
import type { SnapshotEnvironment } from '../types/environment';
export declare function testNameToKey(testName: string, count: number): string;
export declare function keyToTestName(key: string): string;
export declare function getSnapshotData(content: string | null, options: SnapshotStateOptions): {
    data: SnapshotData;
    dirty: boolean;
};
export declare function addExtraLineBreaks(string: string): string;
export declare function removeExtraLineBreaks(string: string): string;
export declare function serialize(val: unknown, indent?: number, formatOverrides?: PrettyFormatOptions): string;
export declare function minify(val: unknown): string;
export declare function deserializeString(stringified: string): string;
export declare function escapeBacktickString(str: string): string;
export declare function normalizeNewlines(string: string): string;
export declare function saveSnapshotFile(environment: SnapshotEnvironment, snapshotData: SnapshotData, snapshotPath: string): Promise<void>;
export declare function saveSnapshotFileRaw(environment: SnapshotEnvironment, content: string, snapshotPath: string): Promise<void>;
/**
 * Deep merge, but considers asymmetric matchers. Unlike base util's deep merge,
 * will merge any object-like instance.
 * Compatible with Jest's snapshot matcher. Should not be used outside of snapshot.
 *
 * @example
 * ```ts
 * toMatchSnapshot({
 *   name: expect.stringContaining('text')
 * })
 * ```
 */
export declare function deepMergeSnapshot(target: any, source: any): any;
export declare class DefaultMap<K, V> extends Map<K, V> {
    private defaultFn;
    constructor(defaultFn: (key: K) => V, entries?: Iterable<readonly [K, V]>);
    get(key: K): V;
}
export declare class CounterMap<K> extends DefaultMap<K, number> {
    constructor();
    _total: number | undefined;
    valueOf(): number;
    increment(key: K): void;
    total(): number;
}
