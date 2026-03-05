//#region tests/rolldown-plugin-dts/inline-import-partial-needed/dep.d.ts
interface Keep {
  x: string;
}
declare class Missing {
  y: number;
}
//#endregion
//#region tests/rolldown-plugin-dts/inline-import-partial-needed/index.d.ts
declare function f(): Missing;
//#endregion
export { type Keep, f };
