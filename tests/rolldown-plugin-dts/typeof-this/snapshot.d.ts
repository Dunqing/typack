// index.d.ts
//#region tests/fixtures/typeof-this/index.d.ts
declare class Test {
  functionOne(foo: string, bar: number): void;
  functionTwo(...args: Parameters<typeof this.functionOne>): void;
}
//#endregion
export { Test };