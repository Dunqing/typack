import "./shared/binding-LZxTDF5S.mjs";
import { C as Plugin, Ft as MaybePromise } from "./shared/define-config-BH0LmADa.mjs";

//#region src/plugin/parallel-plugin-implementation.d.ts
type ParallelPluginImplementation = Plugin;
type Context = {
  /**
  * Thread number
  */
  threadNumber: number;
};
declare function defineParallelPluginImplementation<Options>(plugin: (Options: Options, context: Context) => MaybePromise<ParallelPluginImplementation>): (Options: Options, context: Context) => MaybePromise<ParallelPluginImplementation>;
//#endregion
export { type Context, type ParallelPluginImplementation, defineParallelPluginImplementation };