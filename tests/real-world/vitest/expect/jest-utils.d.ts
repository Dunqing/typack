import type { StandardSchemaV1 } from '@standard-schema/spec';
import type { AsymmetricMatcher } from './jest-asymmetric-matchers';
import type { Tester } from './types';
export declare function equals(a: unknown, b: unknown, customTesters?: Array<Tester>, strictCheck?: boolean): boolean;
export declare function isAsymmetric(obj: any): obj is AsymmetricMatcher<any>;
export declare function hasAsymmetric(obj: any, seen?: Set<any>): boolean;
export declare function isError(value: unknown): value is Error;
export declare function isA(typeName: string, value: unknown): boolean;
export declare function fnNameFor(func: Function): string;
export declare function hasProperty(obj: object | null, property: string): boolean;
export declare function isImmutableUnorderedKeyed(maybeKeyed: any): boolean;
export declare function isImmutableUnorderedSet(maybeSet: any): boolean;
export declare function iterableEquality(a: any, b: any, customTesters?: Array<Tester>, aStack?: Array<any>, bStack?: Array<any>): boolean | undefined;
export declare function subsetEquality(object: unknown, subset: unknown, customTesters?: Array<Tester>): boolean | undefined;
export declare function typeEquality(a: any, b: any): boolean | undefined;
export declare function arrayBufferEquality(a: unknown, b: unknown): boolean | undefined;
export declare function sparseArrayEquality(a: unknown, b: unknown, customTesters?: Array<Tester>): boolean | undefined;
export declare function generateToBeMessage(deepEqualityName: string, expected?: string, actual?: string): string;
export declare function pluralize(word: string, count: number): string;
export declare function getObjectKeys(object: object): Array<string | symbol>;
export declare function getObjectSubset(object: any, subset: any, customTesters: Array<Tester>): {
    subset: any;
    stripped: number;
};
/**
 * Detects if an object is a Standard Schema V1 compatible schema
 */
export declare function isStandardSchema(obj: any): obj is StandardSchemaV1;
