//#region tests/rolldown-plugin-dts/inline-import/bar.d.ts
interface Bar {}
declare const Baz = 123;
//#endregion
//#region tests/rolldown-plugin-dts/inline-import/index.d.ts
export interface Foo {
  bar: Bar;
  baz: typeof Baz;
}
//#endregion
