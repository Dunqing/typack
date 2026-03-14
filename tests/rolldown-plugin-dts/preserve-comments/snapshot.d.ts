//#region tests/rolldown-plugin-dts/preserve-comments/first.d.ts
/**
* A function with doc-comment that is imported first
*/
declare function first(): void;
//#endregion
//#region tests/rolldown-plugin-dts/preserve-comments/second.d.ts
/**
* A function with doc-comment that is imported second
*/
declare function second(): void;
//#endregion
//#region tests/rolldown-plugin-dts/preserve-comments/index.d.ts
/**
* A function with doc-comment in the main file
*/
export declare function main(): void;
//#endregion
export { first, second };
