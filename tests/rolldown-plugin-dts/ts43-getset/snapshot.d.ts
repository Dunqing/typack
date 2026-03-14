//#region tests/rolldown-plugin-dts/ts43-getset/foo.d.ts
interface GetT {}
interface SetT {}
//#endregion
//#region tests/rolldown-plugin-dts/ts43-getset/index.d.ts
export interface Thing {
  get size(): GetT;
  set size(value: GetT | SetT | boolean);
}
//#endregion
