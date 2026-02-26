import { ParserPlugin, parse as babelParse } from "@babel/parser";
import MagicString from "magic-string";
import { BindingMetadata, CodegenResult, CompilerError, CompilerOptions, ParserOptions, RawSourceMap, RootNode, SourceLocation, extractIdentifiers, generateCodeFrame, isInDestructureAssignment, isStaticProperty, walkIdentifiers } from "@vue/compiler-core";
import * as _babel_types from "@babel/types";
import { CallExpression, Expression, Node, ObjectPattern, Program, Statement, TSCallSignatureDeclaration, TSFunctionType, TSMethodSignature, TSModuleDeclaration, TSPropertySignature, TSType } from "@babel/types";
import TS from "typescript";
import { LazyResult, Result } from "postcss";

//#region tests/real-world/core/compiler-sfc/template/transformAssetUrl.d.ts
interface AssetURLTagConfig {
  [name: string]: string[];
}
interface AssetURLOptions {
  /**
  * If base is provided, instead of transforming relative asset urls into
  * imports, they will be directly rewritten to absolute urls.
  */
  base?: string | null;
  /**
  * If true, also processes absolute urls.
  */
  includeAbsolute?: boolean;
  tags?: AssetURLTagConfig;
}
//#endregion
//#region tests/real-world/core/compiler-sfc/compileTemplate.d.ts
interface TemplateCompiler {
  compile(source: string | RootNode, options: CompilerOptions): CodegenResult;
  parse(template: string, options: ParserOptions): RootNode;
}
interface SFCTemplateCompileResults {
  code: string;
  ast?: RootNode;
  preamble?: string;
  source: string;
  tips: string[];
  errors: (string | CompilerError)[];
  map?: RawSourceMap;
}
interface SFCTemplateCompileOptions {
  source: string;
  ast?: RootNode;
  filename: string;
  id: string;
  scoped?: boolean;
  slotted?: boolean;
  isProd?: boolean;
  ssr?: boolean;
  ssrCssVars?: string[];
  inMap?: RawSourceMap;
  compiler?: TemplateCompiler;
  compilerOptions?: CompilerOptions;
  preprocessLang?: string;
  preprocessOptions?: any;
  /**
  * In some cases, compiler-sfc may not be inside the project root (e.g. when
  * linked or globally installed). In such cases a custom `require` can be
  * passed to correctly resolve the preprocessors.
  */
  preprocessCustomRequire?: (id: string) => any;
  /**
  * Configure what tags/attributes to transform into asset url imports,
  * or disable the transform altogether with `false`.
  */
  transformAssetUrls?: AssetURLOptions | AssetURLTagConfig | boolean;
}
declare function compileTemplate(options: SFCTemplateCompileOptions): SFCTemplateCompileResults;
//#endregion
//#region tests/real-world/core/compiler-sfc/compileScript.d.ts
interface SFCScriptCompileOptions {
  /**
  * Scope ID for prefixing injected CSS variables.
  * This must be consistent with the `id` passed to `compileStyle`.
  */
  id: string;
  /**
  * Production mode. Used to determine whether to generate hashed CSS variables
  */
  isProd?: boolean;
  /**
  * Enable/disable source map. Defaults to true.
  */
  sourceMap?: boolean;
  /**
  * https://babeljs.io/docs/en/babel-parser#plugins
  */
  babelParserPlugins?: ParserPlugin[];
  /**
  * A list of files to parse for global types to be made available for type
  * resolving in SFC macros. The list must be fully resolved file system paths.
  */
  globalTypeFiles?: string[];
  /**
  * Compile the template and inline the resulting render function
  * directly inside setup().
  * - Only affects `<script setup>`
  * - This should only be used in production because it prevents the template
  * from being hot-reloaded separately from component state.
  */
  inlineTemplate?: boolean;
  /**
  * Generate the final component as a variable instead of default export.
  * This is useful in e.g. @vitejs/plugin-vue where the script needs to be
  * placed inside the main module.
  */
  genDefaultAs?: string;
  /**
  * Options for template compilation when inlining. Note these are options that
  * would normally be passed to `compiler-sfc`'s own `compileTemplate()`, not
  * options passed to `compiler-dom`.
  */
  templateOptions?: Partial<SFCTemplateCompileOptions>;
  /**
  * Hoist <script setup> static constants.
  * - Only enables when one `<script setup>` exists.
  * @default true
  */
  hoistStatic?: boolean;
  /**
  * Set to `false` to disable reactive destructure for `defineProps` (pre-3.5
  * behavior), or set to `'error'` to throw hard error on props destructures.
  * @default true
  */
  propsDestructure?: boolean | "error";
  /**
  * File system access methods to be used when resolving types
  * imported in SFC macros. Defaults to ts.sys in Node.js, can be overwritten
  * to use a virtual file system for use in browsers (e.g. in REPLs)
  */
  fs?: {
    fileExists(file: string): boolean;
    readFile(file: string): string | undefined;
    realpath?(file: string): string;
  };
  /**
  * Transform Vue SFCs into custom elements.
  */
  customElement?: boolean | ((filename: string) => boolean);
}
interface ImportBinding {
  isType: boolean;
  imported: string;
  local: string;
  source: string;
  isFromSetup: boolean;
  isUsedInTemplate: boolean;
}
/**
* Compile `<script setup>`
* It requires the whole SFC descriptor because we need to handle and merge
* normal `<script>` + `<script setup>` if both are present.
*/
declare function compileScript(sfc: SFCDescriptor, options: SFCScriptCompileOptions): SFCScriptBlock;
//#endregion
//#region tests/real-world/core/compiler-sfc/parse.d.ts
interface SFCParseOptions {
  filename?: string;
  sourceMap?: boolean;
  sourceRoot?: string;
  pad?: boolean | "line" | "space";
  ignoreEmpty?: boolean;
  compiler?: TemplateCompiler;
  templateParseOptions?: ParserOptions;
}
interface SFCBlock {
  type: string;
  content: string;
  attrs: Record<string, string | true>;
  loc: SourceLocation;
  map?: RawSourceMap;
  lang?: string;
  src?: string;
}
interface SFCTemplateBlock extends SFCBlock {
  type: "template";
  ast?: RootNode;
}
interface SFCScriptBlock extends SFCBlock {
  type: "script";
  setup?: string | boolean;
  bindings?: BindingMetadata;
  imports?: Record<string, ImportBinding>;
  scriptAst?: _babel_types.Statement[];
  scriptSetupAst?: _babel_types.Statement[];
  warnings?: string[];
  /**
  * Fully resolved dependency file paths (unix slashes) with imported types
  * used in macros, used for HMR cache busting in @vitejs/plugin-vue and
  * vue-loader.
  */
  deps?: string[];
}
interface SFCStyleBlock extends SFCBlock {
  type: "style";
  scoped?: boolean;
  module?: string | boolean;
}
interface SFCDescriptor {
  filename: string;
  source: string;
  template: SFCTemplateBlock | null;
  script: SFCScriptBlock | null;
  scriptSetup: SFCScriptBlock | null;
  styles: SFCStyleBlock[];
  customBlocks: SFCBlock[];
  cssVars: string[];
  /**
  * whether the SFC uses :slotted() modifier.
  * this is used as a compiler optimization hint.
  */
  slotted: boolean;
  /**
  * compare with an existing descriptor to determine whether HMR should perform
  * a reload vs. re-render.
  *
  * Note: this comparison assumes the prev/next script are already identical,
  * and only checks the special case where <script setup lang="ts"> unused import
  * pruning result changes due to template changes.
  */
  shouldForceReload: (prevImports: Record<string, ImportBinding>) => boolean;
}
interface SFCParseResult {
  descriptor: SFCDescriptor;
  errors: (CompilerError | SyntaxError)[];
}
declare function parse(source: string, options?: SFCParseOptions): SFCParseResult;
//#endregion
//#region tests/real-world/core/compiler-sfc/style/preprocessors.d.ts
type PreprocessLang = "less" | "sass" | "scss" | "styl" | "stylus";
//#endregion
//#region tests/real-world/core/compiler-sfc/compileStyle.d.ts
interface SFCStyleCompileOptions {
  source: string;
  filename: string;
  id: string;
  scoped?: boolean;
  trim?: boolean;
  isProd?: boolean;
  inMap?: RawSourceMap;
  preprocessLang?: PreprocessLang;
  preprocessOptions?: any;
  preprocessCustomRequire?: (id: string) => any;
  postcssOptions?: any;
  postcssPlugins?: any[];
  /**
  * @deprecated use `inMap` instead.
  */
  map?: RawSourceMap;
}
/**
* Aligns with postcss-modules
* https://github.com/css-modules/postcss-modules
*/
interface CSSModulesOptions {
  scopeBehaviour?: "global" | "local";
  generateScopedName?: string | ((name: string, filename: string, css: string) => string);
  hashPrefix?: string;
  localsConvention?: "camelCase" | "camelCaseOnly" | "dashes" | "dashesOnly";
  exportGlobals?: boolean;
  globalModulePaths?: RegExp[];
}
interface SFCAsyncStyleCompileOptions extends SFCStyleCompileOptions {
  isAsync?: boolean;
  modules?: boolean;
  modulesOptions?: CSSModulesOptions;
}
interface SFCStyleCompileResults {
  code: string;
  map: RawSourceMap | undefined;
  rawResult: Result | LazyResult | undefined;
  errors: Error[];
  modules?: Record<string, string>;
  dependencies: Set<string>;
}
declare function compileStyle(options: SFCStyleCompileOptions): SFCStyleCompileResults;
declare function compileStyleAsync(options: SFCAsyncStyleCompileOptions): Promise<SFCStyleCompileResults>;
//#endregion
//#region tests/real-world/core/compiler-sfc/rewriteDefault.d.ts
declare function rewriteDefault(input: string, as: string, parserPlugins?: ParserPlugin[]): string;
/**
* Utility for rewriting `export default` in a script block into a variable
* declaration so that we can inject things into it
*/
declare function rewriteDefaultAST(ast: Statement[], s: MagicString, as: string): void;
//#endregion
//#region tests/real-world/core/compiler-sfc/script/defineProps.d.ts
type PropsDestructureBindings = Record<string, {
  local: string;
  default?: Expression;
}>;
declare function extractRuntimeProps(ctx: TypeResolveContext): string | undefined;
//#endregion
//#region tests/real-world/core/compiler-sfc/script/defineModel.d.ts
interface ModelDecl {
  type: TSType | undefined;
  options: string | undefined;
  identifier: string | undefined;
  runtimeOptionNodes: Node[];
}
//#endregion
//#region tests/real-world/core/compiler-sfc/script/context.d.ts
declare class ScriptCompileContext {
  descriptor: SFCDescriptor;
  options: Partial<SFCScriptCompileOptions>;
  isJS: boolean;
  isTS: boolean;
  isCE: boolean;
  scriptAst: Program | null;
  scriptSetupAst: Program | null;
  source: string;
  filename: string;
  s: MagicString;
  startOffset: number | undefined;
  endOffset: number | undefined;
  scope?: TypeScope;
  globalScopes?: TypeScope[];
  userImports: Record<string, ImportBinding>;
  hasDefinePropsCall: boolean;
  hasDefineEmitCall: boolean;
  hasDefineExposeCall: boolean;
  hasDefaultExportName: boolean;
  hasDefaultExportRender: boolean;
  hasDefineOptionsCall: boolean;
  hasDefineSlotsCall: boolean;
  hasDefineModelCall: boolean;
  propsCall: CallExpression | undefined;
  propsDecl: Node | undefined;
  propsRuntimeDecl: Node | undefined;
  propsTypeDecl: Node | undefined;
  propsDestructureDecl: ObjectPattern | undefined;
  propsDestructuredBindings: PropsDestructureBindings;
  propsDestructureRestId: string | undefined;
  propsRuntimeDefaults: Node | undefined;
  emitsRuntimeDecl: Node | undefined;
  emitsTypeDecl: Node | undefined;
  emitDecl: Node | undefined;
  modelDecls: Record<string, ModelDecl>;
  optionsRuntimeDecl: Node | undefined;
  bindingMetadata: BindingMetadata;
  helperImports: Set<string>;
  helper(key: string): string;
  /**
  * to be exposed on compiled script block for HMR cache busting
  */
  deps?: Set<string>;
  /**
  * cache for resolved fs
  */
  fs?: NonNullable<SFCScriptCompileOptions["fs"]>;
  constructor(descriptor: SFCDescriptor, options: Partial<SFCScriptCompileOptions>);
  getString(node: Node, scriptSetup?: boolean): string;
  warn(msg: string, node: Node, scope?: TypeScope): void;
  error(msg: string, node: Node, scope?: TypeScope): never;
}
//#endregion
//#region tests/real-world/core/compiler-sfc/script/resolveType.d.ts
type SimpleTypeResolveOptions = Partial<Pick<SFCScriptCompileOptions, "globalTypeFiles" | "fs" | "babelParserPlugins" | "isProd">>;
/**
* TypeResolveContext is compatible with ScriptCompileContext
* but also allows a simpler version of it with minimal required properties
* when resolveType needs to be used in a non-SFC context, e.g. in a babel
* plugin. The simplest context can be just:
* ```ts
* const ctx: SimpleTypeResolveContext = {
*   filename: '...',
*   source: '...',
*   options: {},
*   error() {},
*   ast: []
* }
* ```
*/
type SimpleTypeResolveContext = Pick<ScriptCompileContext, "source" | "filename" | "error" | "warn" | "helper" | "getString" | "propsTypeDecl" | "propsRuntimeDefaults" | "propsDestructuredBindings" | "emitsTypeDecl" | "isCE"> & Partial<Pick<ScriptCompileContext, "scope" | "globalScopes" | "deps" | "fs">> & {
  ast: Statement[];
  options: SimpleTypeResolveOptions;
};
type TypeResolveContext = (ScriptCompileContext | SimpleTypeResolveContext) & {
  silentOnExtendsFailure?: boolean;
};
type Import = Pick<ImportBinding, "source" | "imported">;
interface WithScope {
  _ownerScope: TypeScope;
}
type ScopeTypeNode = Node & WithScope & {
  _ns?: TSModuleDeclaration & WithScope;
};
declare class TypeScope {
  filename: string;
  source: string;
  offset: number;
  imports: Record<string, Import>;
  types: Record<string, ScopeTypeNode>;
  declares: Record<string, ScopeTypeNode>;
  constructor(filename: string, source: string, offset?: number, imports?: Record<string, Import>, types?: Record<string, ScopeTypeNode>, declares?: Record<string, ScopeTypeNode>);
  isGenericScope: boolean;
  resolvedImportSources: Record<string, string>;
  exportedTypes: Record<string, ScopeTypeNode>;
  exportedDeclares: Record<string, ScopeTypeNode>;
}
interface MaybeWithScope {
  _ownerScope?: TypeScope;
}
interface ResolvedElements {
  props: Record<string, (TSPropertySignature | TSMethodSignature) & {
    _ownerScope: TypeScope;
  }>;
  calls?: (TSCallSignatureDeclaration | TSFunctionType)[];
}
/**
* Resolve arbitrary type node to a list of type elements that can be then
* mapped to runtime props or emits.
*/
declare function resolveTypeElements(ctx: TypeResolveContext, node: Node & MaybeWithScope & {
  _resolvedElements?: ResolvedElements;
}, scope?: TypeScope, typeParameters?: Record<string, Node>): ResolvedElements;
/**
* @private
*/
declare function registerTS(_loadTS: () => typeof TS): void;
/**
* @private
*/
declare function invalidateTypeCache(filename: string): void;
declare function inferRuntimeType(ctx: TypeResolveContext, node: Node & MaybeWithScope, scope?: TypeScope, isKeyOf?: boolean, typeParameters?: Record<string, Node>): string[];
//#endregion
//#region tests/real-world/core/compiler-sfc/script/defineEmits.d.ts
declare function extractRuntimeEmits(ctx: TypeResolveContext): Set<string>;
//#endregion
//#region tests/real-world/core/compiler-sfc.d.ts
declare const version: string;
declare const parseCache$1: Map<string, SFCParseResult>;
declare const errorMessages: Record<number, string>;
declare const walk: any;
/**
* @deprecated this is preserved to avoid breaking vite-plugin-vue < 5.0
* with reactivityTransform: true. The desired behavior should be silently
* ignoring the option instead of breaking.
*/
declare const shouldTransformRef: () => boolean;
//#endregion
export { type AssetURLOptions, type AssetURLTagConfig, type BindingMetadata, type CompilerError, type CompilerOptions, MagicString, type SFCAsyncStyleCompileOptions, type SFCBlock, type SFCDescriptor, type SFCParseOptions, type SFCParseResult, type SFCScriptBlock, type SFCScriptCompileOptions, type SFCStyleBlock, type SFCStyleCompileOptions, type SFCStyleCompileResults, type SFCTemplateBlock, type SFCTemplateCompileOptions, type SFCTemplateCompileResults, type ScriptCompileContext, type SimpleTypeResolveContext, type SimpleTypeResolveOptions, type TemplateCompiler, type TypeResolveContext, babelParse, compileScript, compileStyle, compileStyleAsync, compileTemplate, errorMessages, extractIdentifiers, extractRuntimeEmits, extractRuntimeProps, generateCodeFrame, inferRuntimeType, invalidateTypeCache, isInDestructureAssignment, isStaticProperty, parse, parseCache$1 as parseCache, registerTS, resolveTypeElements, rewriteDefault, rewriteDefaultAST, shouldTransformRef, version, walk, walkIdentifiers };