/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { DiffOptions, DiffOptionsNormalized } from './types';
import { Diff } from './cleanupSemantic';
export declare function printDiffLines(diffs: Array<Diff>, truncated: boolean, options: DiffOptionsNormalized): string;
export declare function diffLinesUnified(aLines: Array<string>, bLines: Array<string>, options?: DiffOptions): string;
export declare function diffLinesUnified2(aLinesDisplay: Array<string>, bLinesDisplay: Array<string>, aLinesCompare: Array<string>, bLinesCompare: Array<string>, options?: DiffOptions): string;
export declare function diffLinesRaw(aLines: Array<string>, bLines: Array<string>, options?: DiffOptions): [Array<Diff>, boolean];
