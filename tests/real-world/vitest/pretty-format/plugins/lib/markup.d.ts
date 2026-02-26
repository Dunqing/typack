/**
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
import type { Config, Printer, Refs } from '../../types';
export declare function printProps(keys: Array<string>, props: Record<string, unknown>, config: Config, indentation: string, depth: number, refs: Refs, printer: Printer): string;
export declare function printChildren(children: Array<unknown>, config: Config, indentation: string, depth: number, refs: Refs, printer: Printer): string;
export declare function printShadowRoot(children: Array<unknown>, config: Config, indentation: string, depth: number, refs: Refs, printer: Printer): string;
export declare function printText(text: string, config: Config): string;
export declare function printComment(comment: string, config: Config): string;
export declare function printElement(type: string, printedProps: string, printedChildren: string, config: Config, indentation: string): string;
export declare function printElementAsLeaf(type: string, config: Config): string;
