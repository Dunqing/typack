/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { Diff } from './cleanupSemantic';
import type { DiffOptions } from './types';
export declare function diffStringsUnified(a: string, b: string, options?: DiffOptions): string;
export declare function diffStringsRaw(a: string, b: string, cleanup: boolean, options?: DiffOptions): [Array<Diff>, boolean];
