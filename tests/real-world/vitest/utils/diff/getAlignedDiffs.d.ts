/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { DiffOptionsColor } from './types';
import { Diff } from './cleanupSemantic';
declare function getAlignedDiffs(diffs: Array<Diff>, changeColor: DiffOptionsColor): Array<Diff>;
export default getAlignedDiffs;
