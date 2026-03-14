//#region tests/rolldown-plugin-dts/ts43-staticindex/foo.d.ts
interface StaticT {}
//#endregion
//#region tests/rolldown-plugin-dts/ts43-staticindex/index.d.ts
export declare class Foo {
  static hello: string;
  static world: number;
  [propName: string]: string | number | StaticT;
}
//#endregion
