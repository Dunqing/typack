//#region tests/fixtures/star-reexport-local-export-alias/dep.d.ts
interface Internal {
  value: string;
}
//#endregion
export { Internal as Public };
