type ValueType = 'array' | 'bigint' | 'boolean' | 'function' | 'null' | 'number' | 'object' | 'regexp' | 'map' | 'set' | 'date' | 'string' | 'symbol' | 'undefined';
export declare function getType(value: unknown): ValueType;
export declare const isPrimitive: (value: unknown) => boolean;
export {};
