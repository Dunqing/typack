//#region tests/rolldown-plugin-dts/remove-private/index.d.ts
declare class B {}
export declare class Foo {
  private a;
  protected b: B;
  private ma;
  protected mb(): void;
}
//#endregion
