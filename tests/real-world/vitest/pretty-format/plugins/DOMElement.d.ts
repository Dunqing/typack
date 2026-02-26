/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { Config, NewPlugin } from '../types';
export declare const test: NewPlugin['test'];
export interface FilterConfig extends Config {
    filterNode?: (node: any) => boolean;
}
export declare const serialize: NewPlugin['serialize'];
export declare function createDOMElementFilter(filterNode?: (node: any) => boolean): NewPlugin;
declare const plugin: NewPlugin;
export default plugin;
