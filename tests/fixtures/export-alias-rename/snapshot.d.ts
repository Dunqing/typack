//#region tests/fixtures/export-alias-rename/types.d.ts
interface MyType {
  value: string;
}
//#endregion
//#region tests/fixtures/export-alias-rename/index.d.ts
export interface Foo extends MyType {
  extra: number;
}
//#endregion
