/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { NewPlugin } from '../types';
export interface ReactTestObject {
    $$typeof: symbol;
    type: string;
    props?: Record<string, unknown>;
    children?: null | Array<ReactTestChild>;
}
type ReactTestChild = ReactTestObject | string | number;
export declare const serialize: NewPlugin['serialize'];
export declare const test: NewPlugin['test'];
declare const plugin: NewPlugin;
export default plugin;
