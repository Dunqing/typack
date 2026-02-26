type Key = string | symbol;
export type CreateMockInstanceProcedure = (options?: {
    prototypeMembers?: (string | symbol)[];
    name?: string | symbol;
    originalImplementation?: (...args: any[]) => any;
    keepMembersImplementation?: boolean;
}) => any;
export interface MockObjectOptions {
    type: 'automock' | 'autospy';
    globalConstructors: GlobalConstructors;
    createMockInstance: CreateMockInstanceProcedure;
}
export declare function mockObject(options: MockObjectOptions, object: Record<Key, any>, mockExports?: Record<Key, any>): Record<Key, any>;
export interface GlobalConstructors {
    Object: ObjectConstructor;
    Function: FunctionConstructor;
    RegExp: RegExpConstructor;
    Array: ArrayConstructor;
    Map: MapConstructor;
}
export {};
