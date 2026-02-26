/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { DiffOptions } from './types';
import { Diff, DIFF_DELETE, DIFF_EQUAL, DIFF_INSERT } from './cleanupSemantic';
import { diffLinesRaw, diffLinesUnified, diffLinesUnified2 } from './diffLines';
import { diffStringsRaw, diffStringsUnified } from './printDiffs';
export type { DiffOptions, DiffOptionsColor, SerializedDiffOptions } from './types';
export { diffLinesRaw, diffLinesUnified, diffLinesUnified2 };
export { diffStringsRaw, diffStringsUnified };
export { Diff, DIFF_DELETE, DIFF_EQUAL, DIFF_INSERT };
/**
 * @param a Expected value
 * @param b Received value
 * @param options Diff options
 * @returns {string | null} a string diff
 */
export declare function diff(a: any, b: any, options?: DiffOptions): string | undefined;
export declare function printDiffOrStringify(received: unknown, expected: unknown, options?: DiffOptions): string | undefined;
export declare function replaceAsymmetricMatcher(actual: any, expected: any, actualReplaced?: WeakSet<WeakKey>, expectedReplaced?: WeakSet<WeakKey>): {
    replacedActual: any;
    replacedExpected: any;
};
type PrintLabel = (string: string) => string;
export declare function getLabelPrinter(...strings: Array<string>): PrintLabel;
