import { p as BindingReplacePluginConfig } from "./shared/binding-LZxTDF5S.mjs";
import { M as BuiltinPlugin } from "./shared/define-config-BH0LmADa.mjs";
import { t as esmExternalRequirePlugin } from "./shared/constructors-D9d481Vd.mjs";

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