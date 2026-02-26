// index.d.ts
//#region tests/fixtures/export-alias-rename/types.d.ts
interface MyType {
  value: string;
}
//#endregion
//#region tests/fixtures/export-alias-rename/index.d.ts
interface Foo extends MyType {
  extra: number;
}
//#endregion
export { Foo };
