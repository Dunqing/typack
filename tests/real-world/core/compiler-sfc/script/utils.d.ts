import type { CallExpression, Expression, Identifier, ImportDefaultSpecifier, ImportNamespaceSpecifier, ImportSpecifier, Node, StringLiteral, TSMethodSignature, TSPropertySignature } from '@babel/types';
export declare const UNKNOWN_TYPE = "Unknown";
export declare function resolveObjectKey(node: Node, computed: boolean): string | undefined;
export declare function concatStrings(strs: Array<string | null | undefined | false>): string;
export declare function isLiteralNode(node: Node): boolean;
export declare function isCallOf(node: Node | null | undefined, test: string | ((id: string) => boolean) | null | undefined): node is CallExpression;
export declare function toRuntimeTypeString(types: string[]): string;
export declare function getImportedName(specifier: ImportSpecifier | ImportDefaultSpecifier | ImportNamespaceSpecifier): string;
export declare function getId(node: Identifier | StringLiteral): string;
export declare function getId(node: Expression): string | null;
export declare function getStringLiteralKey(node: TSPropertySignature | TSMethodSignature): string | null;
/**
 * We need `getCanonicalFileName` when creating ts module resolution cache,
 * but TS does not expose it directly. This implementation is replicated from
 * the TS source code.
 */
export declare function createGetCanonicalFileName(useCaseSensitiveFileNames: boolean): (str: string) => string;
export declare function normalizePath(p: string): string;
export declare const joinPaths: (...paths: string[]) => string;
/**
 * key may contain symbols
 * e.g. onUpdate:modelValue -> "onUpdate:modelValue"
 */
export declare const propNameEscapeSymbolsRE: RegExp;
export declare function getEscapedPropName(key: string): string;
export declare const isJS: (...langs: (string | null | undefined)[]) => boolean;
export declare const isTS: (...langs: (string | null | undefined)[]) => boolean;
