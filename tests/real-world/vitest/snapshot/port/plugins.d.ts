/**
 * Copyright (c) Facebook, Inc. and its affiliates. All Rights Reserved.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { Plugin as PrettyFormatPlugin, Plugins as PrettyFormatPlugins } from '@vitest/pretty-format';
export declare function addSerializer(plugin: PrettyFormatPlugin): void;
export declare function getSerializers(): PrettyFormatPlugins;
