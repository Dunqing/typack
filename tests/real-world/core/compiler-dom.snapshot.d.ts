import { CodegenResult, CompilerError, CompilerOptions, DirectiveTransform, NodeTransform, ParserOptions, RootNode, SourceLocation } from "@vue/compiler-core";
export * from "@vue/compiler-core";

//#region tests/real-world/core/compiler-dom/parserOptions.d.ts
declare const parserOptions: ParserOptions;
//#endregion
//#region tests/real-world/core/compiler-dom/runtimeHelpers.d.ts
declare const V_MODEL_RADIO: unique symbol;
declare const V_MODEL_CHECKBOX: unique symbol;
declare const V_MODEL_TEXT: unique symbol;
declare const V_MODEL_SELECT: unique symbol;
declare const V_MODEL_DYNAMIC: unique symbol;
declare const V_ON_WITH_MODIFIERS: unique symbol;
declare const V_ON_WITH_KEYS: unique symbol;
declare const V_SHOW: unique symbol;
declare const TRANSITION: unique symbol;
declare const TRANSITION_GROUP: unique symbol;
//#endregion
//#region tests/real-world/core/compiler-dom/transforms/transformStyle.d.ts
declare const transformStyle: NodeTransform;
//#endregion
//#region tests/real-world/core/compiler-dom/errors.d.ts
interface DOMCompilerError extends CompilerError {
  code: DOMErrorCodes;
}
declare function createDOMCompilerError(code: DOMErrorCodes, loc?: SourceLocation): DOMCompilerError;
declare enum DOMErrorCodes {
  X_V_HTML_NO_EXPRESSION = 54,
  X_V_HTML_WITH_CHILDREN = 55,
  X_V_TEXT_NO_EXPRESSION = 56,
  X_V_TEXT_WITH_CHILDREN = 57,
  X_V_MODEL_ON_INVALID_ELEMENT = 58,
  X_V_MODEL_ARG_ON_ELEMENT = 59,
  X_V_MODEL_ON_FILE_INPUT_ELEMENT = 60,
  X_V_MODEL_UNNECESSARY_VALUE = 61,
  X_V_SHOW_NO_EXPRESSION = 62,
  X_TRANSITION_INVALID_CHILDREN = 63,
  X_IGNORED_SIDE_EFFECT_TAG = 64,
  __EXTEND_POINT__ = 65
}
declare const DOMErrorMessages: {
  [code: number]: string;
};
//#endregion
//#region tests/real-world/core/compiler-dom.d.ts
export declare const DOMNodeTransforms: NodeTransform[];
export declare const DOMDirectiveTransforms: Record<string, DirectiveTransform>;
export declare function compile(src: string | RootNode, options?: CompilerOptions): CodegenResult;
export declare function parse(template: string, options?: ParserOptions): RootNode;
//#endregion
export { DOMErrorCodes, DOMErrorMessages, TRANSITION, TRANSITION_GROUP, V_MODEL_CHECKBOX, V_MODEL_DYNAMIC, V_MODEL_RADIO, V_MODEL_SELECT, V_MODEL_TEXT, V_ON_WITH_KEYS, V_ON_WITH_MODIFIERS, V_SHOW, createDOMCompilerError, parserOptions, transformStyle };