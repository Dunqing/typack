//#region tests/rolldown-plugin-dts/type-this/index.d.ts
declare class Foo {
  a: this;
}
export declare function thisType(this: Foo): void;
//#endregion
