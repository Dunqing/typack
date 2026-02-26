import { type Target } from './reactive';
declare class BaseReactiveHandler implements ProxyHandler<Target> {
    protected readonly _isReadonly: boolean;
    protected readonly _isShallow: boolean;
    constructor(_isReadonly?: boolean, _isShallow?: boolean);
    get(target: Target, key: string | symbol, receiver: object): any;
}
declare class MutableReactiveHandler extends BaseReactiveHandler {
    constructor(isShallow?: boolean);
    set(target: Record<string | symbol, unknown>, key: string | symbol, value: unknown, receiver: object): boolean;
    deleteProperty(target: Record<string | symbol, unknown>, key: string | symbol): boolean;
    has(target: Record<string | symbol, unknown>, key: string | symbol): boolean;
    ownKeys(target: Record<string | symbol, unknown>): (string | symbol)[];
}
declare class ReadonlyReactiveHandler extends BaseReactiveHandler {
    constructor(isShallow?: boolean);
    set(target: object, key: string | symbol): boolean;
    deleteProperty(target: object, key: string | symbol): boolean;
}
export declare const mutableHandlers: ProxyHandler<object>;
export declare const readonlyHandlers: ProxyHandler<object>;
export declare const shallowReactiveHandlers: MutableReactiveHandler;
export declare const shallowReadonlyHandlers: ReadonlyReactiveHandler;
export {};
