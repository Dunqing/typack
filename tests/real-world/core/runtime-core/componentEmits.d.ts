import { type OverloadParameters, type UnionToIntersection } from '@vue/shared';
import { type ComponentInternalInstance, type ConcreteComponent } from './component';
import type { AppContext } from './apiCreateApp';
import type { ComponentTypeEmits } from './apiSetupHelpers';
import type { ComponentPublicInstance } from './componentPublicInstance';
export type ObjectEmitsOptions = Record<string, ((...args: any[]) => any) | null>;
export type EmitsOptions = ObjectEmitsOptions | string[];
export type EmitsToProps<T extends EmitsOptions | ComponentTypeEmits> = T extends string[] ? {
    [K in `on${Capitalize<T[number]>}`]?: (...args: any[]) => any;
} : T extends ObjectEmitsOptions ? {
    [K in string & keyof T as `on${Capitalize<K>}`]?: (...args: T[K] extends (...args: infer P) => any ? P : T[K] extends null ? any[] : never) => any;
} : {};
export type TypeEmitsToOptions<T extends ComponentTypeEmits> = {
    [K in keyof T & string]: T[K] extends [...args: infer Args] ? (...args: Args) => any : () => any;
} & (T extends (...args: any[]) => any ? ParametersToFns<OverloadParameters<T>> : {});
type ParametersToFns<T extends any[]> = {
    [K in T[0]]: IsStringLiteral<K> extends true ? (...args: T extends [e: infer E, ...args: infer P] ? K extends E ? P : never : never) => any : never;
};
type IsStringLiteral<T> = T extends string ? string extends T ? false : true : false;
export type ShortEmitsToObject<E> = E extends Record<string, any[]> ? {
    [K in keyof E]: (...args: E[K]) => any;
} : E;
export type EmitFn<Options = ObjectEmitsOptions, Event extends keyof Options = keyof Options> = Options extends Array<infer V> ? (event: V, ...args: any[]) => void : {} extends Options ? (event: string, ...args: any[]) => void : UnionToIntersection<{
    [key in Event]: Options[key] extends (...args: infer Args) => any ? (event: key, ...args: Args) => void : Options[key] extends any[] ? (event: key, ...args: Options[key]) => void : (event: key, ...args: any[]) => void;
}[Event]>;
export declare function emit(instance: ComponentInternalInstance, event: string, ...rawArgs: any[]): ComponentPublicInstance | null | undefined;
export declare function normalizeEmitsOptions(comp: ConcreteComponent, appContext: AppContext, asMixin?: boolean): ObjectEmitsOptions | null;
export declare function isEmitListener(options: ObjectEmitsOptions | null, key: string): boolean;
export {};
