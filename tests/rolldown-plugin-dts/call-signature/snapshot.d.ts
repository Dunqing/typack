//#region tests/rolldown-plugin-dts/call-signature/index.d.ts
export interface I {
  (arg: string): string;
  staticProp: string;
}
export declare const fn: {
  (arg: string): string;
  staticProp: string;
};
//#endregion
