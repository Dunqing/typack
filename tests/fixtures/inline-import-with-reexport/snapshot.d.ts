//#region tests/fixtures/inline-import-with-reexport/dep.d.ts
interface Keep {
  kept: true;
}
interface Missing {
  found: true;
}
//#endregion
//#region tests/fixtures/inline-import-with-reexport/index.d.ts
export type Found = Missing;
//#endregion
export { type Keep };
