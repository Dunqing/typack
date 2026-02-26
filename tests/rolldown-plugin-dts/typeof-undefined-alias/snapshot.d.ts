// index.d.ts
//#region tests/fixtures/typeof-undefined-alias/index.d.ts
declare let undefined$1: string;
type T = typeof undefined$1;
//#endregion
export { T, undefined$1 as undefined };