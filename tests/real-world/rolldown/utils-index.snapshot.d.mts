import { Program } from "@oxc-project/types";

//#region tests/real-world/rolldown/shared/binding-LZxTDF5S.d.mts
interface CodegenOptions {
  /**
  * Remove whitespace.
  *
  * @default true
  */
  removeWhitespace?: boolean;
}
interface CompressOptions {
  /**
  * Set desired EcmaScript standard version for output.
  *
  * Set `esnext` to enable all target highering.
  *
  * Example:
  *
  * * `'es2015'`
  * * `['es2020', 'chrome58', 'edge16', 'firefox57', 'node12', 'safari11']`
  *
  * @default 'esnext'
  *
  * @see [esbuild#target](https://esbuild.github.io/api/#target)
  */
  target?: string | Array<string>;
  /**
  * Pass true to discard calls to `console.*`.
  *
  * @default false
  */
  dropConsole?: boolean;
  /**
  * Remove `debugger;` statements.
  *
  * @default true
  */
  dropDebugger?: boolean;
  /**
  * Pass `true` to drop unreferenced functions and variables.
  *
  * Simple direct variable assignments do not count as references unless set to `keep_assign`.
  * @default true
  */
  unused?: boolean | "keep_assign";
  /** Keep function / class names. */
  keepNames?: CompressOptionsKeepNames;
  /**
  * Join consecutive var, let and const statements.
  *
  * @default true
  */
  joinVars?: boolean;
  /**
  * Join consecutive simple statements using the comma operator.
  *
  * `a; b` -> `a, b`
  *
  * @default true
  */
  sequences?: boolean;
  /**
  * Set of label names to drop from the code.
  *
  * Labeled statements matching these names will be removed during minification.
  *
  * @default []
  */
  dropLabels?: Array<string>;
  /** Limit the maximum number of iterations for debugging purpose. */
  maxIterations?: number;
  /** Treeshake options. */
  treeshake?: TreeShakeOptions;
}
interface CompressOptionsKeepNames {
  /**
  * Keep function names so that `Function.prototype.name` is preserved.
  *
  * This does not guarantee that the `undefined` name is preserved.
  *
  * @default false
  */
  function: boolean;
  /**
  * Keep class names so that `Class.prototype.name` is preserved.
  *
  * This does not guarantee that the `undefined` name is preserved.
  *
  * @default false
  */
  class: boolean;
}
interface MangleOptions {
  /**
  * Pass `true` to mangle names declared in the top level scope.
  *
  * @default true for modules and commonjs, otherwise false
  */
  toplevel?: boolean;
  /**
  * Preserve `name` property for functions and classes.
  *
  * @default false
  */
  keepNames?: boolean | MangleOptionsKeepNames;
  /** Debug mangled names. */
  debug?: boolean;
}
interface MangleOptionsKeepNames {
  /**
  * Preserve `name` property for functions.
  *
  * @default false
  */
  function: boolean;
  /**
  * Preserve `name` property for classes.
  *
  * @default false
  */
  class: boolean;
}
interface MinifyOptions$1 {
  /** Use when minifying an ES module. */
  module?: boolean;
  compress?: boolean | CompressOptions;
  mangle?: boolean | MangleOptions;
  codegen?: boolean | CodegenOptions;
  sourcemap?: boolean;
}
interface MinifyResult {
  code: string;
  map?: SourceMap;
  errors: Array<OxcError>;
}
interface TreeShakeOptions {
  /**
  * Whether to respect the pure annotations.
  *
  * Pure annotations are comments that mark an expression as pure.
  * For example: @__PURE__ or #__NO_SIDE_EFFECTS__.
  *
  * @default true
  */
  annotations?: boolean;
  /**
  * Whether to treat this function call as pure.
  *
  * This function is called for normal function calls, new calls, and
  * tagged template calls.
  */
  manualPureFunctions?: Array<string>;
  /**
  * Whether property read accesses have side effects.
  *
  * @default 'always'
  */
  propertyReadSideEffects?: boolean | "always";
  /**
  * Whether accessing a global variable has side effects.
  *
  * Accessing a non-existing global variable will throw an error.
  * Global variable may be a getter that has side effects.
  *
  * @default true
  */
  unknownGlobalSideEffects?: boolean;
  /**
  * Whether invalid import statements have side effects.
  *
  * Accessing a non-existing import name will throw an error.
  * Also import statements that cannot be resolved will throw an error.
  *
  * @default true
  */
  invalidImportSideEffects?: boolean;
}
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
interface SourceMap {
  file?: string;
  mappings: string;
  names: Array<string>;
  sourceRoot?: string;
  sources: Array<string>;
  sourcesContent?: Array<string>;
  version: number;
  x_google_ignoreList?: Array<number>;
}
interface CompilerAssumptions {
  ignoreFunctionLength?: boolean;
  noDocumentAll?: boolean;
  objectRestNoSymbols?: boolean;
  pureGetters?: boolean;
  /**
  * When using public class fields, assume that they don't shadow any getter in the current class,
  * in its subclasses or in its superclass. Thus, it's safe to assign them rather than using
  * `Object.defineProperty`.
  *
  * For example:
  *
  * Input:
  * ```js
  * class Test {
  *  field = 2;
  *
  *  static staticField = 3;
  * }
  * ```
  *
  * When `set_public_class_fields` is `true`, the output will be:
  * ```js
  * class Test {
  *  constructor() {
  *    this.field = 2;
  *  }
  * }
  * Test.staticField = 3;
  * ```
  *
  * Otherwise, the output will be:
  * ```js
  * import _defineProperty from "@oxc-project/runtime/helpers/defineProperty";
  * class Test {
  *   constructor() {
  *     _defineProperty(this, "field", 2);
  *   }
  * }
  * _defineProperty(Test, "staticField", 3);
  * ```
  *
  * NOTE: For TypeScript, if you wanted behavior is equivalent to `useDefineForClassFields: false`, you should
  * set both `set_public_class_fields` and [`crate::TypeScriptOptions::remove_class_fields_without_initializer`]
  * to `true`.
  */
  setPublicClassFields?: boolean;
}
interface DecoratorOptions {
  /**
  * Enables experimental support for decorators, which is a version of decorators that predates the TC39 standardization process.
  *
  * Decorators are a language feature which hasn’t yet been fully ratified into the JavaScript specification.
  * This means that the implementation version in TypeScript may differ from the implementation in JavaScript when it it decided by TC39.
  *
  * @see https://www.typescriptlang.org/tsconfig/#experimentalDecorators
  * @default false
  */
  legacy?: boolean;
  /**
  * Enables emitting decorator metadata.
  *
  * This option the same as [emitDecoratorMetadata](https://www.typescriptlang.org/tsconfig/#emitDecoratorMetadata)
  * in TypeScript, and it only works when `legacy` is true.
  *
  * @see https://www.typescriptlang.org/tsconfig/#emitDecoratorMetadata
  * @default false
  */
  emitDecoratorMetadata?: boolean;
}
type HelperMode = "Runtime" | "External";
interface Helpers {
  mode?: HelperMode;
}
interface IsolatedDeclarationsOptions {
  /**
  * Do not emit declarations for code that has an @internal annotation in its JSDoc comment.
  * This is an internal compiler option; use at your own risk, because the compiler does not check that the result is valid.
  *
  * Default: `false`
  *
  * See <https://www.typescriptlang.org/tsconfig/#stripInternal>
  */
  stripInternal?: boolean;
  sourcemap?: boolean;
}
/**
* Configure how TSX and JSX are transformed.
*
* @see {@link https://oxc.rs/docs/guide/usage/transformer/jsx}
*/
interface JsxOptions {
  /**
  * Decides which runtime to use.
  *
  * - 'automatic' - auto-import the correct JSX factories
  * - 'classic' - no auto-import
  *
  * @default 'automatic'
  */
  runtime?: "classic" | "automatic";
  /**
  * Emit development-specific information, such as `__source` and `__self`.
  *
  * @default false
  */
  development?: boolean;
  /**
  * Toggles whether or not to throw an error if an XML namespaced tag name
  * is used.
  *
  * Though the JSX spec allows this, it is disabled by default since React's
  * JSX does not currently have support for it.
  *
  * @default true
  */
  throwIfNamespace?: boolean;
  /**
  * Mark JSX elements and top-level React method calls as pure for tree shaking.
  *
  * @default true
  */
  pure?: boolean;
  /**
  * Replaces the import source when importing functions.
  *
  * @default 'react'
  */
  importSource?: string;
  /**
  * Replace the function used when compiling JSX expressions. It should be a
  * qualified name (e.g. `React.createElement`) or an identifier (e.g.
  * `createElement`).
  *
  * Only used for `classic` {@link runtime}.
  *
  * @default 'React.createElement'
  */
  pragma?: string;
  /**
  * Replace the component used when compiling JSX fragments. It should be a
  * valid JSX tag name.
  *
  * Only used for `classic` {@link runtime}.
  *
  * @default 'React.Fragment'
  */
  pragmaFrag?: string;
  /**
  * Enable React Fast Refresh .
  *
  * Conforms to the implementation in {@link https://github.com/facebook/react/tree/v18.3.1/packages/react-refresh}
  *
  * @default false
  */
  refresh?: boolean | ReactRefreshOptions;
}
interface PluginsOptions {
  styledComponents?: StyledComponentsOptions;
  taggedTemplateEscape?: boolean;
}
interface ReactRefreshOptions {
  /**
  * Specify the identifier of the refresh registration variable.
  *
  * @default `$RefreshReg$`.
  */
  refreshReg?: string;
  /**
  * Specify the identifier of the refresh signature variable.
  *
  * @default `$RefreshSig$`.
  */
  refreshSig?: string;
  emitFullSignatures?: boolean;
}
/**
* Configure how styled-components are transformed.
*
* @see {@link https://oxc.rs/docs/guide/usage/transformer/plugins#styled-components}
*/
interface StyledComponentsOptions {
  /**
  * Enhances the attached CSS class name on each component with richer output to help
  * identify your components in the DOM without React DevTools.
  *
  * @default true
  */
  displayName?: boolean;
  /**
  * Controls whether the `displayName` of a component will be prefixed with the filename
  * to make the component name as unique as possible.
  *
  * @default true
  */
  fileName?: boolean;
  /**
  * Adds a unique identifier to every styled component to avoid checksum mismatches
  * due to different class generation on the client and server during server-side rendering.
  *
  * @default true
  */
  ssr?: boolean;
  /**
  * Transpiles styled-components tagged template literals to a smaller representation
  * than what Babel normally creates, helping to reduce bundle size.
  *
  * @default true
  */
  transpileTemplateLiterals?: boolean;
  /**
  * Minifies CSS content by removing all whitespace and comments from your CSS,
  * keeping valuable bytes out of your bundles.
  *
  * @default true
  */
  minify?: boolean;
  /**
  * Enables transformation of JSX `css` prop when using styled-components.
  *
  * **Note: This feature is not yet implemented in oxc.**
  *
  * @default true
  */
  cssProp?: boolean;
  /**
  * Enables "pure annotation" to aid dead code elimination by bundlers.
  *
  * @default false
  */
  pure?: boolean;
  /**
  * Adds a namespace prefix to component identifiers to ensure class names are unique.
  *
  * Example: With `namespace: "my-app"`, generates `componentId: "my-app__sc-3rfj0a-1"`
  */
  namespace?: string;
  /**
  * List of file names that are considered meaningless for component naming purposes.
  *
  * When the `fileName` option is enabled and a component is in a file with a name
  * from this list, the directory name will be used instead of the file name for
  * the component's display name.
  *
  * @default `["index"]`
  */
  meaninglessFileNames?: Array<string>;
  /**
  * Import paths to be considered as styled-components imports at the top level.
  *
  * **Note: This feature is not yet implemented in oxc.**
  */
  topLevelImportPaths?: Array<string>;
}
interface TypeScriptOptions {
  jsxPragma?: string;
  jsxPragmaFrag?: string;
  onlyRemoveTypeImports?: boolean;
  allowNamespaces?: boolean;
  /**
  * When enabled, type-only class fields are only removed if they are prefixed with the declare modifier:
  *
  * @deprecated
  *
  * Allowing `declare` fields is built-in support in Oxc without any option. If you want to remove class fields
  * without initializer, you can use `remove_class_fields_without_initializer: true` instead.
  */
  allowDeclareFields?: boolean;
  /**
  * When enabled, class fields without initializers are removed.
  *
  * For example:
  * ```ts
  * class Foo {
  *    x: number;
  *    y: number = 0;
  * }
  * ```
  * // transform into
  * ```js
  * class Foo {
  *    x: number;
  * }
  * ```
  *
  * The option is used to align with the behavior of TypeScript's `useDefineForClassFields: false` option.
  * When you want to enable this, you also need to set [`crate::CompilerAssumptions::set_public_class_fields`]
  * to `true`. The `set_public_class_fields: true` + `remove_class_fields_without_initializer: true` is
  * equivalent to `useDefineForClassFields: false` in TypeScript.
  *
  * When `set_public_class_fields` is true and class-properties plugin is enabled, the above example transforms into:
  *
  * ```js
  * class Foo {
  *   constructor() {
  *     this.y = 0;
  *   }
  * }
  * ```
  *
  * Defaults to `false`.
  */
  removeClassFieldsWithoutInitializer?: boolean;
  /**
  * Also generate a `.d.ts` declaration file for TypeScript files.
  *
  * The source file must be compliant with all
  * [`isolatedDeclarations`](https://www.typescriptlang.org/docs/handbook/release-notes/typescript-5-5.html#isolated-declarations)
  * requirements.
  *
  * @default false
  */
  declaration?: IsolatedDeclarationsOptions;
  /**
  * Rewrite or remove TypeScript import/export declaration extensions.
  *
  * - When set to `rewrite`, it will change `.ts`, `.mts`, `.cts` extensions to `.js`, `.mjs`, `.cjs` respectively.
  * - When set to `remove`, it will remove `.ts`/`.mts`/`.cts`/`.tsx` extension entirely.
  * - When set to `true`, it's equivalent to `rewrite`.
  * - When set to `false` or omitted, no changes will be made to the extensions.
  *
  * @default false
  */
  rewriteImportExtensions?: "rewrite" | "remove" | boolean;
}
/**
* Cache for tsconfig resolution to avoid redundant file system operations.
*
* The cache stores resolved tsconfig configurations keyed by their file paths.
* When transforming multiple files in the same project, tsconfig lookups are
* deduplicated, improving performance.
*/
declare class TsconfigCache {
  /** Create a new transform cache with auto tsconfig discovery enabled. */
  constructor();
  /**
  * Clear the cache.
  *
  * Call this when tsconfig files have changed to ensure fresh resolution.
  */
  clear(): void;
  /** Get the number of cached entries. */
  size(): number;
}
/** Enhanced transform options with tsconfig and inputMap support. */
interface BindingEnhancedTransformOptions {
  /** Treat the source text as 'js', 'jsx', 'ts', 'tsx', or 'dts'. */
  lang?: "js" | "jsx" | "ts" | "tsx" | "dts";
  /** Treat the source text as 'script', 'module', 'commonjs', or 'unambiguous'. */
  sourceType?: "script" | "module" | "commonjs" | "unambiguous" | undefined;
  /**
  * The current working directory. Used to resolve relative paths in other
  * options.
  */
  cwd?: string;
  /**
  * Enable source map generation.
  *
  * When `true`, the `sourceMap` field of transform result objects will be populated.
  *
  * @default false
  */
  sourcemap?: boolean;
  /** Set assumptions in order to produce smaller output. */
  assumptions?: CompilerAssumptions;
  /**
  * Configure how TypeScript is transformed.
  * @see {@link https://oxc.rs/docs/guide/usage/transformer/typescript}
  */
  typescript?: TypeScriptOptions;
  /**
  * Configure how TSX and JSX are transformed.
  * @see {@link https://oxc.rs/docs/guide/usage/transformer/jsx}
  */
  jsx?: "preserve" | JsxOptions;
  /**
  * Sets the target environment for the generated JavaScript.
  *
  * The lowest target is `es2015`.
  *
  * Example:
  *
  * * `'es2015'`
  * * `['es2020', 'chrome58', 'edge16', 'firefox57', 'node12', 'safari11']`
  *
  * @default `esnext` (No transformation)
  *
  * @see {@link https://oxc.rs/docs/guide/usage/transformer/lowering#target}
  */
  target?: string | Array<string>;
  /** Behaviour for runtime helpers. */
  helpers?: Helpers;
  /**
  * Define Plugin
  * @see {@link https://oxc.rs/docs/guide/usage/transformer/global-variable-replacement#define}
  */
  define?: Record<string, string>;
  /**
  * Inject Plugin
  * @see {@link https://oxc.rs/docs/guide/usage/transformer/global-variable-replacement#inject}
  */
  inject?: Record<string, string | [string, string]>;
  /** Decorator plugin */
  decorator?: DecoratorOptions;
  /**
  * Third-party plugins to use.
  * @see {@link https://oxc.rs/docs/guide/usage/transformer/plugins}
  */
  plugins?: PluginsOptions;
  /**
  * Configure tsconfig handling.
  * - true: Auto-discover and load the nearest tsconfig.json
  * - TsconfigRawOptions: Use the provided inline tsconfig options
  */
  tsconfig?: boolean | BindingTsconfigRawOptions;
  /** An input source map to collapse with the output source map. */
  inputMap?: SourceMap;
}
/** Result of the enhanced transform API. */
interface BindingEnhancedTransformResult {
  /**
  * The transformed code.
  *
  * If parsing failed, this will be an empty string.
  */
  code: string;
  /**
  * The source map for the transformed code.
  *
  * This will be set if {@link BindingEnhancedTransformOptions#sourcemap} is `true`.
  */
  map?: SourceMap;
  /**
  * The `.d.ts` declaration file for the transformed code. Declarations are
  * only generated if `declaration` is set to `true` and a TypeScript file
  * is provided.
  *
  * If parsing failed and `declaration` is set, this will be an empty string.
  *
  * @see {@link TypeScriptOptions#declaration}
  * @see [declaration tsconfig option](https://www.typescriptlang.org/tsconfig/#declaration)
  */
  declaration?: string;
  /**
  * Declaration source map. Only generated if both
  * {@link TypeScriptOptions#declaration declaration} and
  * {@link BindingEnhancedTransformOptions#sourcemap sourcemap} are set to `true`.
  */
  declarationMap?: SourceMap;
  /**
  * Helpers used.
  *
  * @internal
  *
  * Example:
  *
  * ```text
  * { "_objectSpread": "@oxc-project/runtime/helpers/objectSpread2" }
  * ```
  */
  helpersUsed: Record<string, string>;
  /** Parse and transformation errors. */
  errors: Array<BindingError>;
  /** Parse and transformation warnings. */
  warnings: Array<BindingError>;
  /** Paths to tsconfig files that were loaded during transformation. */
  tsconfigFilePaths: Array<string>;
}
type BindingError = {
  type: "JsError";
  field0: Error;
} | {
  type: "NativeError";
  field0: NativeError;
};
interface BindingLogLocation {
  /** 1-based */
  line: number;
  /** 0-based position in the line in UTF-16 code units */
  column: number;
  file?: string;
}
/** TypeScript compiler options for inline tsconfig configuration. */
interface BindingTsconfigCompilerOptions {
  /** Specifies the JSX factory function to use. */
  jsx?: "react" | "react-jsx" | "react-jsxdev" | "preserve" | "react-native";
  /** Specifies the JSX factory function. */
  jsxFactory?: string;
  /** Specifies the JSX fragment factory function. */
  jsxFragmentFactory?: string;
  /** Specifies the module specifier for JSX imports. */
  jsxImportSource?: string;
  /** Enables experimental decorators. */
  experimentalDecorators?: boolean;
  /** Enables decorator metadata emission. */
  emitDecoratorMetadata?: boolean;
  /** Preserves module structure of imports/exports. */
  verbatimModuleSyntax?: boolean;
  /** Configures how class fields are emitted. */
  useDefineForClassFields?: boolean;
  /** The ECMAScript target version. */
  target?: string;
  /** @deprecated Use verbatimModuleSyntax instead. */
  preserveValueImports?: boolean;
  /** @deprecated Use verbatimModuleSyntax instead. */
  importsNotUsedAsValues?: "remove" | "preserve" | "error";
}
/** Raw tsconfig options for inline configuration. */
interface BindingTsconfigRawOptions {
  /** TypeScript compiler options. */
  compilerOptions?: BindingTsconfigCompilerOptions;
}
/** Error emitted from native side, it only contains kind and message, no stack trace. */
interface NativeError {
  kind: string;
  message: string;
  /** The id of the file associated with the error */
  id?: string;
  /** The exporter associated with the error (for import/export errors) */
  exporter?: string;
  /** Location information (line, column, file) */
  loc?: BindingLogLocation;
  /** Position in the source file in UTF-16 code units */
  pos?: number;
}
//#endregion
//#region tests/real-world/rolldown/shared/logging-CKYae7lu.d.mts
interface RolldownLog {
  binding?: string;
  cause?: unknown;
  /**
  * The log code for this log object.
  * @example 'PLUGIN_ERROR'
  */
  code?: string;
  exporter?: string;
  frame?: string;
  hook?: string;
  id?: string;
  ids?: string[];
  loc?: {
    column: number;
    file?: string;
    line: number;
  };
  /**
  * The message for this log object.
  * @example 'The "transform" hook used by the output plugin "rolldown-plugin-foo" is a build time hook and will not be run for that plugin. Either this plugin cannot be used as an output plugin, or it should have an option to configure it as an output plugin.'
  */
  message: string;
  meta?: any;
  names?: string[];
  plugin?: string;
  pluginCode?: unknown;
  pos?: number;
  reexporter?: string;
  stack?: string;
  url?: string;
}
//#endregion
//#region tests/real-world/rolldown/shared/transform-DSI3Dv-p.d.mts
//#region src/utils/parse.d.ts
/**
* Parse JS/TS source asynchronously on a separate thread.
*
* Note that not all of the workload can happen on a separate thread.
* Parsing on Rust side does happen in a separate thread, but deserialization of the AST to JS objects
* has to happen on current thread. This synchronous deserialization work typically outweighs
* the asynchronous parsing by a factor of between 3 and 20.
*
* i.e. the majority of the workload cannot be parallelized by using this method.
*
* Generally `parseSync` is preferable to use as it does not have the overhead of spawning a thread.
* If you need to parallelize parsing multiple files, it is recommended to use worker threads.
*/
declare function parse(filename: string, sourceText: string, options?: ParserOptions | null): Promise<ParseResult>;
/**
* Parse JS/TS source synchronously on current thread.
*
* This is generally preferable over `parse` (async) as it does not have the overhead
* of spawning a thread, and the majority of the workload cannot be parallelized anyway
* (see `parse` documentation for details).
*
* If you need to parallelize parsing multiple files, it is recommended to use worker threads
* with `parseSync` rather than using `parse`.
*/
declare function parseSync(filename: string, sourceText: string, options?: ParserOptions | null): ParseResult;
//#endregion
//#region src/utils/minify.d.ts
type MinifyOptions = MinifyOptions$1 & {
  inputMap?: SourceMap;
};
/**
* Minify asynchronously.
*
* Note: This function can be slower than `minifySync` due to the overhead of spawning a thread.
*
* @experimental
*/
declare function minify(filename: string, sourceText: string, options?: MinifyOptions | null): Promise<MinifyResult>;
/**
* Minify synchronously.
*
* @experimental
*/
declare function minifySync(filename: string, sourceText: string, options?: MinifyOptions | null): MinifyResult;
//#endregion
//#region src/utils/transform.d.ts
type TransformResult = Omit<BindingEnhancedTransformResult, "errors" | "warnings"> & {
  errors: Error[];
  warnings: RolldownLog[];
};
/**
* Transpile a JavaScript or TypeScript into a target ECMAScript version, asynchronously.
*
* Note: This function can be slower than `transformSync` due to the overhead of spawning a thread.
*
* @param filename The name of the file being transformed. If this is a
* relative path, consider setting the {@link TransformOptions#cwd} option.
* @param source_text The source code to transform.
* @param options The transform options including tsconfig and inputMap. See {@link
* BindingEnhancedTransformOptions} for more information.
* @param cache Optional tsconfig cache for reusing resolved tsconfig across multiple transforms.
* Only used when `options.tsconfig` is `true`.
*
* @returns a promise that resolves to an object containing the transformed code,
* source maps, and any errors that occurred during parsing or transformation.
*
* @experimental
*/
declare function transform(filename: string, sourceText: string, options?: BindingEnhancedTransformOptions | null, cache?: TsconfigCache | null): Promise<TransformResult>;
/**
* Transpile a JavaScript or TypeScript into a target ECMAScript version.
*
* @param filename The name of the file being transformed. If this is a
* relative path, consider setting the {@link TransformOptions#cwd} option.
* @param source_text The source code to transform.
* @param options The transform options including tsconfig and inputMap. See {@link
* BindingEnhancedTransformOptions} for more information.
* @param cache Optional tsconfig cache for reusing resolved tsconfig across multiple transforms.
* Only used when `options.tsconfig` is `true`.
*
* @returns an object containing the transformed code, source maps, and any errors
* that occurred during parsing or transformation.
*
* @experimental
*/
declare function transformSync(filename: string, sourceText: string, options?: BindingEnhancedTransformOptions | null, cache?: TsconfigCache | null): TransformResult;
//#endregion
export { type MinifyOptions, type MinifyResult, type ParseResult, type ParserOptions, type BindingEnhancedTransformOptions as TransformOptions, type TransformResult, TsconfigCache, type BindingTsconfigCompilerOptions as TsconfigCompilerOptions, type BindingTsconfigRawOptions as TsconfigRawOptions, minify, minifySync, parse, parseSync, transform, transformSync };