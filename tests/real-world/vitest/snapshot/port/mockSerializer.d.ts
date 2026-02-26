/**
 * Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 *
 * https://github.com/facebook/jest/blob/4eb4f6a59b6eae0e05b8e51dd8cd3fdca1c7aff1/packages/jest-snapshot/src/mockSerializer.ts#L4
 */
import type { NewPlugin } from '@vitest/pretty-format';
export declare const serialize: NewPlugin['serialize'];
export declare const test: NewPlugin['test'];
declare const plugin: NewPlugin;
export default plugin;
