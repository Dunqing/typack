//#region tests/real-world/rolldown/shared/binding-LZxTDF5S.d.mts
type BindingStringOrRegex = string | RegExp;
type BindingBuiltinPluginName = "builtin:esm-external-require" | "builtin:isolated-declaration" | "builtin:replace" | "builtin:vite-alias" | "builtin:vite-build-import-analysis" | "builtin:vite-dynamic-import-vars" | "builtin:vite-import-glob" | "builtin:vite-json" | "builtin:vite-load-fallback" | "builtin:vite-manifest" | "builtin:vite-module-preload-polyfill" | "builtin:vite-react-refresh-wrapper" | "builtin:vite-reporter" | "builtin:vite-resolve" | "builtin:vite-transform" | "builtin:vite-wasm-fallback" | "builtin:vite-web-worker-post";
interface BindingEsmExternalRequirePluginConfig {
  external: Array<BindingStringOrRegex>;
  skipDuplicateCheck?: boolean;
}
interface BindingReplacePluginConfig {
  values: Record<string, string>;
  delimiters?: [string, string];
  preventAssignment?: boolean;
  objectGuards?: boolean;
  sourcemap?: boolean;
}
//#endregion
//#region tests/real-world/rolldown/shared/define-config-BH0LmADa.d.mts
//#endregion
//#region src/builtin-plugin/utils.d.ts
declare class BuiltinPlugin {
  name: BindingBuiltinPluginName;
  _options?: unknown;
  /** Vite-specific option to control plugin ordering */
  enforce?: "pre" | "post";
  constructor(name: BindingBuiltinPluginName, _options?: unknown);
}
//#endregion
//#region tests/real-world/rolldown/shared/constructors-D9d481Vd.d.mts
declare function esmExternalRequirePlugin(config?: BindingEsmExternalRequirePluginConfig): BuiltinPlugin;
//#endregion
//#region tests/real-world/rolldown/plugins-index.d.mts
//#region src/builtin-plugin/replace-plugin.d.ts
/**
* Replaces targeted strings in files while bundling.
*
* @example
* // Basic usage
* ```js
* replacePlugin({
*   'process.env.NODE_ENV': JSON.stringify('production'),
*    __buildVersion: 15
* })
* ```
* @example
* // With options
* ```js
* replacePlugin({
*   'process.env.NODE_ENV': JSON.stringify('production'),
*   __buildVersion: 15
* }, {
*   preventAssignment: false,
* })
* ```
*/
declare function replacePlugin(values?: BindingReplacePluginConfig["values"], options?: Omit<BindingReplacePluginConfig, "values">): BuiltinPlugin;
//#endregion
export { esmExternalRequirePlugin, replacePlugin };