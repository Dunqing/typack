//#region tests/rolldown-plugin-dts/circular-to-entry/Foo.d.ts
declare class Foo {
  manager: FooManager;
  constructor(manager: FooManager);
}
//#endregion
//#region tests/rolldown-plugin-dts/circular-to-entry/index.d.ts
declare class FooManager {
  foos: Array<Foo>;
  constructor();
}
//#endregion
export { FooManager as default };
