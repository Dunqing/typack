//#region tests/rolldown-plugin-dts/import-renamed/bar.d.ts
interface Bar {}
//#endregion
//#region tests/rolldown-plugin-dts/import-renamed/index.d.ts
export interface Foo {
  bar: Bar;
}
//#endregion
