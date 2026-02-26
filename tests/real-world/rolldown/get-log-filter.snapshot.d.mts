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
//#region tests/real-world/rolldown/get-log-filter.d.mts
//#region src/get-log-filter.d.ts
type GetLogFilter = (filters: string[]) => (log: RolldownLog) => boolean;
declare const getLogFilter: GetLogFilter;
//#endregion
export { GetLogFilter, type RolldownLog, type RolldownLog as RollupLog, getLogFilter as default };