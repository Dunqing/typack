export declare const FRAGMENT: unique symbol;
export declare const TELEPORT: unique symbol;
export declare const SUSPENSE: unique symbol;
export declare const KEEP_ALIVE: unique symbol;
export declare const BASE_TRANSITION: unique symbol;
export declare const OPEN_BLOCK: unique symbol;
export declare const CREATE_BLOCK: unique symbol;
export declare const CREATE_ELEMENT_BLOCK: unique symbol;
export declare const CREATE_VNODE: unique symbol;
export declare const CREATE_ELEMENT_VNODE: unique symbol;
export declare const CREATE_COMMENT: unique symbol;
export declare const CREATE_TEXT: unique symbol;
export declare const CREATE_STATIC: unique symbol;
export declare const RESOLVE_COMPONENT: unique symbol;
export declare const RESOLVE_DYNAMIC_COMPONENT: unique symbol;
export declare const RESOLVE_DIRECTIVE: unique symbol;
export declare const RESOLVE_FILTER: unique symbol;
export declare const WITH_DIRECTIVES: unique symbol;
export declare const RENDER_LIST: unique symbol;
export declare const RENDER_SLOT: unique symbol;
export declare const CREATE_SLOTS: unique symbol;
export declare const TO_DISPLAY_STRING: unique symbol;
export declare const MERGE_PROPS: unique symbol;
export declare const NORMALIZE_CLASS: unique symbol;
export declare const NORMALIZE_STYLE: unique symbol;
export declare const NORMALIZE_PROPS: unique symbol;
export declare const GUARD_REACTIVE_PROPS: unique symbol;
export declare const TO_HANDLERS: unique symbol;
export declare const CAMELIZE: unique symbol;
export declare const CAPITALIZE: unique symbol;
export declare const TO_HANDLER_KEY: unique symbol;
export declare const SET_BLOCK_TRACKING: unique symbol;
/**
 * @deprecated no longer needed in 3.5+ because we no longer hoist element nodes
 * but kept for backwards compat
 */
export declare const PUSH_SCOPE_ID: unique symbol;
/**
 * @deprecated kept for backwards compat
 */
export declare const POP_SCOPE_ID: unique symbol;
export declare const WITH_CTX: unique symbol;
export declare const UNREF: unique symbol;
export declare const IS_REF: unique symbol;
export declare const WITH_MEMO: unique symbol;
export declare const IS_MEMO_SAME: unique symbol;
export declare const helperNameMap: Record<symbol, string>;
export declare function registerRuntimeHelpers(helpers: Record<symbol, string>): void;
