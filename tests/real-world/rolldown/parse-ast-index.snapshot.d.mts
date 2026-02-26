import { Program } from "@oxc-project/types";

//#region tests/real-world/rolldown/shared/binding-LZxTDF5S.d.mts
interface Comment {
  type: "Line" | "Block";
  value: string;
  start: number;
  end: number;
}
interface ErrorLabel {
  message: string | null;
  start: number;
  end: number;
}
interface OxcError {
  severity: Severity;
  message: string;
  labels: Array<ErrorLabel>;
  helpMessage: string | null;
  codeframe: string | null;
}
type Severity = "Error" | "Warning" | "Advice";
declare class ParseResult {
  get program(): Program;
  get module(): EcmaScriptModule;
  get comments(): Array<Comment>;
  get errors(): Array<OxcError>;
}
interface DynamicImport {
  start: number;
  end: number;
  moduleRequest: Span;
}
interface EcmaScriptModule {
  /**
  * Has ESM syntax.
  *
  * i.e. `import` and `export` statements, and `import.meta`.
  *
  * Dynamic imports `import('foo')` are ignored since they can be used in non-ESM files.
  */
  hasModuleSyntax: boolean;
  /** Import statements. */
  staticImports: Array<StaticImport>;
  /** Export statements. */
  staticExports: Array<StaticExport>;
  /** Dynamic import expressions. */
  dynamicImports: Array<DynamicImport>;
  /** Span positions` of `import.meta` */
  importMetas: Array<Span>;
}
interface ExportExportName {
  kind: ExportExportNameKind;
  name: string | null;
  start: number | null;
  end: number | null;
}
type ExportExportNameKind = "Name" | "Default" | "None";
interface ExportImportName {
  kind: ExportImportNameKind;
  name: string | null;
  start: number | null;
  end: number | null;
}
type ExportImportNameKind = "Name" | "All" | "AllButDefault" | "None";
interface ExportLocalName {
  kind: ExportLocalNameKind;
  name: string | null;
  start: number | null;
  end: number | null;
}
type ExportLocalNameKind = "Name" | "Default" | "None";
interface ImportName {
  kind: ImportNameKind;
  name: string | null;
  start: number | null;
  end: number | null;
}
type ImportNameKind = "Name" | "NamespaceObject" | "Default";
interface ParserOptions {
  /** Treat the source text as `js`, `jsx`, `ts`, `tsx` or `dts`. */
  lang?: "js" | "jsx" | "ts" | "tsx" | "dts";
  /** Treat the source text as `script` or `module` code. */
  sourceType?: "script" | "module" | "commonjs" | "unambiguous" | undefined;
  /**
  * Return an AST which includes TypeScript-related properties, or excludes them.
  *
  * `'js'` is default for JS / JSX files.
  * `'ts'` is default for TS / TSX files.
  * The type of the file is determined from `lang` option, or extension of provided `filename`.
  */
  astType?: "js" | "ts";
  /**
  * Controls whether the `range` property is included on AST nodes.
  * The `range` property is a `[number, number]` which indicates the start/end offsets
  * of the node in the file contents.
  *
  * @default false
  */
  range?: boolean;
  /**
  * Emit `ParenthesizedExpression` and `TSParenthesizedType` in AST.
  *
  * If this option is true, parenthesized expressions are represented by
  * (non-standard) `ParenthesizedExpression` and `TSParenthesizedType` nodes that
  * have a single `expression` property containing the expression inside parentheses.
  *
  * @default true
  */
  preserveParens?: boolean;
  /**
  * Produce semantic errors with an additional AST pass.
  * Semantic errors depend on symbols and scopes, where the parser does not construct.
  * This adds a small performance overhead.
  *
  * @default false
  */
  showSemanticErrors?: boolean;
}
interface Span {
  start: number;
  end: number;
}
interface StaticExport {
  start: number;
  end: number;
  entries: Array<StaticExportEntry>;
}
interface StaticExportEntry {
  start: number;
  end: number;
  moduleRequest: ValueSpan | null;
  /** The name under which the desired binding is exported by the module`. */
  importName: ExportImportName;
  /** The name used to export this binding by this module. */
  exportName: ExportExportName;
  /** The name that is used to locally access the exported value from within the importing module. */
  localName: ExportLocalName;
  /**
  * Whether the export is a TypeScript `export type`.
  *
  * Examples:
  *
  * ```ts
  * export type * from 'mod';
  * export type * as ns from 'mod';
  * export type { foo };
  * export { type foo }:
  * export type { foo } from 'mod';
  * ```
  */
  isType: boolean;
}
interface StaticImport {
  /** Start of import statement. */
  start: number;
  /** End of import statement. */
  end: number;
  /**
  * Import source.
  *
  * ```js
  * import { foo } from "mod";
  * //                   ^^^
  * ```
  */
  moduleRequest: ValueSpan;
  /**
  * Import specifiers.
  *
  * Empty for `import "mod"`.
  */
  entries: Array<StaticImportEntry>;
}
interface StaticImportEntry {
  /**
  * The name under which the desired binding is exported by the module.
  *
  * ```js
  * import { foo } from "mod";
  * //       ^^^
  * import { foo as bar } from "mod";
  * //       ^^^
  * ```
  */
  importName: ImportName;
  /**
  * The name that is used to locally access the imported value from within the importing module.
  * ```js
  * import { foo } from "mod";
  * //       ^^^
  * import { foo as bar } from "mod";
  * //              ^^^
  * ```
  */
  localName: ValueSpan;
  /**
  * Whether this binding is for a TypeScript type-only import.
  *
  * `true` for the following imports:
  * ```ts
  * import type { foo } from "mod";
  * import { type foo } from "mod";
  * ```
  */
  isType: boolean;
}
interface ValueSpan {
  value: string;
  start: number;
  end: number;
}
//#endregion
//#region tests/real-world/rolldown/parse-ast-index.d.mts
//#region src/parse-ast-index.d.ts
declare function parseAst(sourceText: string, options?: ParserOptions | null, filename?: string): Program;
declare function parseAstAsync(sourceText: string, options?: ParserOptions | null, filename?: string): Promise<Program>;
//#endregion
export { type ParseResult, type ParserOptions, parseAst, parseAstAsync };