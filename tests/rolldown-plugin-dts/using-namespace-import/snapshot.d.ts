//#region tests/rolldown-plugin-dts/using-namespace-import/namespace.d.ts
interface Bar {}
//#endregion
//#region tests/rolldown-plugin-dts/using-namespace-import/index.d.ts
export interface Foo {
  bar: Bar;
}
//#endregion
