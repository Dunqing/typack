import type { Assertion } from './types';
export declare function createAssertionMessage(util: Chai.ChaiUtils, assertion: Assertion, hasArgs: boolean): string;
export declare function recordAsyncExpect(_test: any, promise: Promise<any>, assertion: string, error: Error, isSoft?: boolean): Promise<any>;
export declare function wrapAssertion(utils: Chai.ChaiUtils, name: string, fn: (this: Chai.AssertionStatic & Assertion, ...args: any[]) => void | PromiseLike<void>): (this: Chai.AssertionStatic & Assertion, ...args: any[]) => void | PromiseLike<void>;
