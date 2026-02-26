/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { NewPlugin, Options, OptionsReceived } from './pretty-format/types';
export { createDOMElementFilter } from './pretty-format/plugins/DOMElement';
export declare const DEFAULT_OPTIONS: Options;
/**
 * Returns a presentation string of your `val` object
 * @param val any potential JavaScript object
 * @param options Custom settings
 */
export declare function format(val: unknown, options?: OptionsReceived): string;
export type { Colors, CompareKeys, Config, NewPlugin, OldPlugin, Options, OptionsReceived, Plugin, Plugins, PrettyFormatOptions, Printer, Refs, Theme, } from './pretty-format/types';
export declare const plugins: {
    AsymmetricMatcher: NewPlugin;
    DOMCollection: NewPlugin;
    DOMElement: NewPlugin;
    Immutable: NewPlugin;
    ReactElement: NewPlugin;
    ReactTestComponent: NewPlugin;
    Error: NewPlugin;
};
