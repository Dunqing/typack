//#region tests/rolldown-plugin-dts/ts43-override/foo.d.ts
interface ShowT {}
interface HideT {}
declare class SomeComponent {
  show(): ShowT;
  hide(): HideT;
}
//#endregion
//#region tests/rolldown-plugin-dts/ts43-override/index.d.ts
export declare class SpecializedComponent extends SomeComponent {
  show(): ShowT;
  hide(): HideT;
}
//#endregion
