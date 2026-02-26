// index.d.ts
declare namespace bar_d_exports {
  export { Bar, IBar };
}
declare class Bar {}
interface IBar {}
//#endregion
//#region tests/fixtures/inline-import-namespace/index.d.ts
interface Foo {
  ns: typeof bar_d_exports;
}
//#endregion
export { Foo };