export declare const CSS_VAR_TEXT: unique symbol;
/**
 * Runtime helper for SFC's CSS variable injection feature.
 * @private
 */
export declare function useCssVars(getter: (ctx: any) => Record<string, unknown>): void;
