/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { Diff } from './cleanupSemantic';
import type { DiffOptionsNormalized } from './types';
export declare function joinAlignedDiffsNoExpand(diffs: Array<Diff>, options: DiffOptionsNormalized): string;
export declare function joinAlignedDiffsExpand(diffs: Array<Diff>, options: DiffOptionsNormalized): string;
