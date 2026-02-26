export declare const isSpecialBooleanAttr: (key: string) => boolean;
/**
 * The full list is needed during SSR to produce the correct initial markup.
 */
export declare const isBooleanAttr: (key: string) => boolean;
/**
 * Boolean attributes should be included if the value is truthy or ''.
 * e.g. `<select multiple>` compiles to `{ multiple: '' }`
 */
export declare function includeBooleanAttr(value: unknown): boolean;
export declare function isSSRSafeAttrName(name: string): boolean;
export declare const propsToAttrMap: Record<string, string | undefined>;
/**
 * Known attributes, this is used for stringification of runtime static nodes
 * so that we don't stringify bindings that cannot be set from HTML.
 * Don't also forget to allow `data-*` and `aria-*`!
 * Generated from https://developer.mozilla.org/en-US/docs/Web/HTML/Attributes
 */
export declare const isKnownHtmlAttr: (key: string) => boolean;
/**
 * Generated from https://developer.mozilla.org/en-US/docs/Web/SVG/Attribute
 */
export declare const isKnownSvgAttr: (key: string) => boolean;
/**
 * Generated from https://developer.mozilla.org/en-US/docs/Web/MathML/Attribute
 */
export declare const isKnownMathMLAttr: (key: string) => boolean;
/**
 * Shared between server-renderer and runtime-core hydration logic
 */
export declare function isRenderableAttrValue(value: unknown): boolean;
