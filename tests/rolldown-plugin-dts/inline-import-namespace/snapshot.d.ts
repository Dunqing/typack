declare namespace bar_d_exports {
  export { Bar, IBar };
}
//#region tests/rolldown-plugin-dts/inline-import-namespace/bar.d.ts
declare class Bar {}
interface IBar {}
//#endregion
//#region tests/rolldown-plugin-dts/inline-import-namespace/index.d.ts
export interface Foo {
  ns: typeof bar_d_exports;
}
//#endregion
