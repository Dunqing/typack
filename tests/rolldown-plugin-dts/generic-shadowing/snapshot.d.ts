//#region tests/rolldown-plugin-dts/generic-shadowing/mod.d.ts
type Config1<Client> = Client;
//#endregion
//#region tests/rolldown-plugin-dts/generic-shadowing/index.d.ts
export type Client = any;
export type Config2<Client> = Client;
//#endregion
export { Config1 };
