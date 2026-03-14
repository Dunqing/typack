import { Node, Token } from "ext-pkg";

//#region tests/fixtures/external-namespace-merge/lib-ns.d.ts
declare function parse(): Node;
//#endregion
//#region tests/fixtures/external-namespace-merge/lib-named.d.ts
interface Lexer {
  current: Token;
}
//#endregion
export { Lexer, parse };
