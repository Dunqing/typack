import { type IfAny } from '@vue/shared';
import { type ComponentInternalInstance, type ConcreteComponent, type Data } from './component';
import type { AppContext } from './apiCreateApp';
export type ComponentPropsOptions<P = Data> = ComponentObjectPropsOptions<P> | string[];
export type ComponentObjectPropsOptions<P = Data> = {
    [K in keyof P]: Prop<P[K]> | null;
};
export type Prop<T, D = T> = PropOptions<T, D> | PropType<T>;
type DefaultFactory<T> = (props: Data) => T | null | undefined;
export interface PropOptions<T = any, D = T> {
    type?: PropType<T> | true | null;
    required?: boolean;
    default?: D | DefaultFactory<D> | null | undefined | object;
    validator?(value: unknown, props: Data): boolean;
}
export type PropType<T> = PropConstructor<T> | (PropConstructor<T> | null)[];
type PropConstructor<T = any> = {
    new (...args: any[]): T & {};
} | {
    (): T;
} | PropMethod<T>;
type PropMethod<T, TConstructor = any> = [T] extends [
    ((...args: any) => any) | undefined
] ? {
    new (): TConstructor;
    (): T;
    readonly prototype: TConstructor;
} : never;
type RequiredKeys<T> = {
    [K in keyof T]: T[K] extends {
        required: true;
    } | {
        default: any;
    } | BooleanConstructor | {
        type: BooleanConstructor;
    } ? T[K] extends {
        default: undefined | (() => undefined);
    } ? never : K : never;
}[keyof T];
type OptionalKeys<T> = Exclude<keyof T, RequiredKeys<T>>;
type DefaultKeys<T> = {
    [K in keyof T]: T[K] extends {
        default: any;
    } | BooleanConstructor | {
        type: BooleanConstructor;
    } ? T[K] extends {
        type: BooleanConstructor;
        required: true;
    } ? never : K : never;
}[keyof T];
type InferPropType<T, NullAsAny = true> = [T] extends [null] ? NullAsAny extends true ? any : null : [T] extends [{
    type: null | true;
}] ? any : [T] extends [ObjectConstructor | {
    type: ObjectConstructor;
}] ? Record<string, any> : [T] extends [BooleanConstructor | {
    type: BooleanConstructor;
}] ? boolean : [T] extends [DateConstructor | {
    type: DateConstructor;
}] ? Date : [T] extends [(infer U)[] | {
    type: (infer U)[];
}] ? U extends DateConstructor ? Date | InferPropType<U, false> : InferPropType<U, false> : [T] extends [Prop<infer V, infer D>] ? unknown extends V ? keyof V extends never ? IfAny<V, V, D> : V : V : T;
/**
 * Extract prop types from a runtime props options object.
 * The extracted types are **internal** - i.e. the resolved props received by
 * the component.
 * - Boolean props are always present
 * - Props with default values are always present
 *
 * To extract accepted props from the parent, use {@link ExtractPublicPropTypes}.
 */
export type ExtractPropTypes<O> = {
    [K in keyof Pick<O, RequiredKeys<O>>]: O[K] extends {
        default: any;
    } ? Exclude<InferPropType<O[K]>, undefined> : InferPropType<O[K]>;
} & {
    [K in keyof Pick<O, OptionalKeys<O>>]?: InferPropType<O[K]>;
};
type PublicRequiredKeys<T> = {
    [K in keyof T]: T[K] extends {
        required: true;
    } ? K : never;
}[keyof T];
type PublicOptionalKeys<T> = Exclude<keyof T, PublicRequiredKeys<T>>;
/**
 * Extract prop types from a runtime props options object.
 * The extracted types are **public** - i.e. the expected props that can be
 * passed to component.
 */
export type ExtractPublicPropTypes<O> = {
    [K in keyof Pick<O, PublicRequiredKeys<O>>]: InferPropType<O[K]>;
} & {
    [K in keyof Pick<O, PublicOptionalKeys<O>>]?: InferPropType<O[K]>;
};
declare enum BooleanFlags {
    shouldCast = 0,
    shouldCastTrue = 1
}
export type ExtractDefaultPropTypes<O> = O extends object ? {
    [K in keyof Pick<O, DefaultKeys<O>>]: InferPropType<O[K]>;
} : {};
type NormalizedProp = PropOptions & {
    [BooleanFlags.shouldCast]?: boolean;
    [BooleanFlags.shouldCastTrue]?: boolean;
};
export type NormalizedProps = Record<string, NormalizedProp>;
export type NormalizedPropsOptions = [NormalizedProps, string[]] | [];
export declare function initProps(instance: ComponentInternalInstance, rawProps: Data | null, isStateful: number, // result of bitwise flag comparison
isSSR?: boolean): void;
export declare function updateProps(instance: ComponentInternalInstance, rawProps: Data | null, rawPrevProps: Data | null, optimized: boolean): void;
export declare function normalizePropsOptions(comp: ConcreteComponent, appContext: AppContext, asMixin?: boolean): NormalizedPropsOptions;
export {};
