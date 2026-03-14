import * as typescript from "typescript";
import * as rollup from "rollup";

//#region tests/rolldown-plugin-dts/inline-import-typeof-members/index.d.ts
export type TypeScript = typeof typescript;
export interface Test {
  rollup: rollup.RollupOptions;
}
//#endregion
