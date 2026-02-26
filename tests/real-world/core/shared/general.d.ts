export declare const EMPTY_OBJ: {
    readonly [key: string]: any;
};
export declare const EMPTY_ARR: readonly never[];
export declare const NOOP: () => void;
/**
 * Always return false.
 */
export declare const NO: () => boolean;
export declare const isOn: (key: string) => boolean;
export declare const isModelListener: (key: string) => key is `onUpdate:${string}`;
export declare const extend: typeof Object.assign;
export declare const remove: <T>(arr: T[], el: T) => void;
export declare const hasOwn: (val: object, key: string | symbol) => key is keyof typeof val;
export declare const isArray: typeof Array.isArray;
export declare const isMap: (val: unknown) => val is Map<any, any>;
export declare const isSet: (val: unknown) => val is Set<any>;
export declare const isDate: (val: unknown) => val is Date;
export declare const isRegExp: (val: unknown) => val is RegExp;
export declare const isFunction: (val: unknown) => val is Function;
export declare const isString: (val: unknown) => val is string;
export declare const isSymbol: (val: unknown) => val is symbol;
export declare const isObject: (val: unknown) => val is Record<any, any>;
export declare const isPromise: <T = any>(val: unknown) => val is Promise<T>;
export declare const objectToString: typeof Object.prototype.toString;
export declare const toTypeString: (value: unknown) => string;
export declare const toRawType: (value: unknown) => string;
export declare const isPlainObject: (val: unknown) => val is object;
export declare const isIntegerKey: (key: unknown) => boolean;
export declare const isReservedProp: (key: string) => boolean;
export declare const isBuiltInDirective: (key: string) => boolean;
/**
 * @private
 */
export declare const camelize: (str: string) => string;
/**
 * @private
 */
export declare const hyphenate: (str: string) => string;
/**
 * @private
 */
export declare const capitalize: <T extends string>(str: T) => Capitalize<T>;
/**
 * @private
 */
export declare const toHandlerKey: <T extends string>(str: T) => T extends '' ? '' : `on${Capitalize<T>}`;
export declare const hasChanged: (value: any, oldValue: any) => boolean;
export declare const invokeArrayFns: (fns: Function[], ...arg: any[]) => void;
export declare const def: (obj: object, key: string | symbol, value: any, writable?: boolean) => void;
/**
 * "123-foo" will be parsed to 123
 * This is used for the .number modifier in v-model
 */
export declare const looseToNumber: (val: any) => any;
/**
 * Only concerns number-like strings
 * "123-foo" will be returned as-is
 */
export declare const toNumber: (val: any) => any;
export declare const getGlobalThis: () => any;
export declare function genPropsAccessExp(name: string): string;
export declare function genCacheKey(source: string, options: any): string;
