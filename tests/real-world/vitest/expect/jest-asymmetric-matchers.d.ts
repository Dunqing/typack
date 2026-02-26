import type { StandardSchemaV1 } from '@standard-schema/spec';
import type { ChaiPlugin, MatcherState, Tester } from './types';
export interface AsymmetricMatcherInterface {
    asymmetricMatch: (other: unknown, customTesters?: Array<Tester>) => boolean;
    toString: () => string;
    getExpectedType?: () => string;
    toAsymmetricMatcher?: () => string;
}
export declare abstract class AsymmetricMatcher<T, State extends MatcherState = MatcherState> implements AsymmetricMatcherInterface {
    protected sample: T;
    protected inverse: boolean;
    $$typeof: symbol;
    constructor(sample: T, inverse?: boolean);
    protected getMatcherContext(expect?: Chai.ExpectStatic): State;
    abstract asymmetricMatch(other: unknown, customTesters?: Array<Tester>): boolean;
    abstract toString(): string;
    getExpectedType?(): string;
    toAsymmetricMatcher?(): string;
}
export declare class StringContaining extends AsymmetricMatcher<string> {
    constructor(sample: string, inverse?: boolean);
    asymmetricMatch(other: string): boolean;
    toString(): string;
    getExpectedType(): string;
}
export declare class Anything extends AsymmetricMatcher<void> {
    asymmetricMatch(other: unknown): boolean;
    toString(): string;
    toAsymmetricMatcher(): string;
}
export declare class ObjectContaining extends AsymmetricMatcher<Record<string | symbol | number, unknown>> {
    constructor(sample: Record<string, unknown>, inverse?: boolean);
    getPrototype(obj: object): any;
    hasProperty(obj: object | null, property: string | symbol): boolean;
    getProperties(obj: object): (string | symbol)[];
    asymmetricMatch(other: any, customTesters?: Array<Tester>): boolean;
    toString(): string;
    getExpectedType(): string;
}
export declare class ArrayContaining<T = unknown> extends AsymmetricMatcher<Array<T>> {
    constructor(sample: Array<T>, inverse?: boolean);
    asymmetricMatch(other: Array<T>, customTesters?: Array<Tester>): boolean;
    toString(): string;
    getExpectedType(): string;
}
export declare class Any extends AsymmetricMatcher<any> {
    constructor(sample: unknown);
    fnNameFor(func: Function): string;
    asymmetricMatch(other: unknown): boolean;
    toString(): string;
    getExpectedType(): string;
    toAsymmetricMatcher(): string;
}
export declare class StringMatching extends AsymmetricMatcher<RegExp> {
    constructor(sample: string | RegExp, inverse?: boolean);
    asymmetricMatch(other: string): boolean;
    toString(): string;
    getExpectedType(): string;
}
export declare class SchemaMatching extends AsymmetricMatcher<StandardSchemaV1<unknown, unknown>> {
    private result;
    constructor(sample: StandardSchemaV1<unknown, unknown>, inverse?: boolean);
    asymmetricMatch(other: unknown): boolean;
    toString(): string;
    getExpectedType(): string;
    toAsymmetricMatcher(): string;
}
export declare const JestAsymmetricMatchers: ChaiPlugin;
