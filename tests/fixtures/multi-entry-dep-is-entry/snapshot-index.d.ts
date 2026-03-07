//#region tests/fixtures/multi-entry-dep-is-entry/utils.d.ts
declare function add(a: number, b: number): number;
//#endregion
//#region tests/fixtures/multi-entry-dep-is-entry/index.d.ts
declare function greet(name: string): string;
//#endregion
export { add, greet };
