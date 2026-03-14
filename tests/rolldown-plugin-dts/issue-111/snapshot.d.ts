//#region tests/rolldown-plugin-dts/issue-111/a.d.ts
interface Stuff {
  id: string;
}
declare const Stuff: Stuff;
//#endregion
//#region tests/rolldown-plugin-dts/issue-111/b.d.ts
interface Stuff$1 {
  id: string;
}
declare const Stuff$1: Stuff$1;
//#endregion
export { Stuff$1 as OtherStuff, Stuff };
