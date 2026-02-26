import { type IfAny, type LooseRequired, type Prettify, type UnionToIntersection } from '@vue/shared';
import { type SetupContext } from './component';
import type { EmitFn, EmitsOptions } from './componentEmits';
import type { ComponentOptionsBase, ComponentOptionsMixin, ComputedOptions, MethodOptions } from './componentOptions';
import type { ComponentObjectPropsOptions, ExtractPropTypes, PropOptions } from './componentProps';
import type { SlotsType, StrictUnwrapSlotsType } from './componentSlots';
import type { Ref } from '@vue/reactivity';
/**
 * Vue `<script setup>` compiler macro for declaring component props. The
 * expected argument is the same as the component `props` option.
 *
 * Example runtime declaration:
 * ```js
 * // using Array syntax
 * const props = defineProps(['foo', 'bar'])
 * // using Object syntax
 * const props = defineProps({
 *   foo: String,
 *   bar: {
 *     type: Number,
 *     required: true
 *   }
 * })
 * ```
 *
 * Equivalent type-based declaration:
 * ```ts
 * // will be compiled into equivalent runtime declarations
 * const props = defineProps<{
 *   foo?: string
 *   bar: number
 * }>()
 * ```
 *
 * @see {@link https://vuejs.org/api/sfc-script-setup.html#defineprops-defineemits}
 *
 * This is only usable inside `<script setup>`, is compiled away in the
 * output and should **not** be actually called at runtime.
 */
export declare function defineProps<PropNames extends string = string>(props: PropNames[]): Prettify<Readonly<{
    [key in PropNames]?: any;
}>>;
export declare function defineProps<PP extends ComponentObjectPropsOptions = ComponentObjectPropsOptions>(props: PP): Prettify<Readonly<ExtractPropTypes<PP>>>;
export declare function defineProps<TypeProps>(): DefineProps<LooseRequired<TypeProps>, BooleanKey<TypeProps>>;
export type DefineProps<T, BKeys extends keyof T> = Readonly<T> & {
    readonly [K in BKeys]-?: boolean;
};
type BooleanKey<T, K extends keyof T = keyof T> = K extends any ? T[K] extends boolean | undefined ? T[K] extends never | undefined ? never : K : never : never;
/**
 * Vue `<script setup>` compiler macro for declaring a component's emitted
 * events. The expected argument is the same as the component `emits` option.
 *
 * Example runtime declaration:
 * ```js
 * const emit = defineEmits(['change', 'update'])
 * ```
 *
 * Example type-based declaration:
 * ```ts
 * const emit = defineEmits<{
 *   // <eventName>: <expected arguments>
 *   change: []
 *   update: [value: number] // named tuple syntax
 * }>()
 *
 * emit('change')
 * emit('update', 1)
 * ```
 *
 * This is only usable inside `<script setup>`, is compiled away in the
 * output and should **not** be actually called at runtime.
 *
 * @see {@link https://vuejs.org/api/sfc-script-setup.html#defineprops-defineemits}
 */
export declare function defineEmits<EE extends string = string>(emitOptions: EE[]): EmitFn<EE[]>;
export declare function defineEmits<E extends EmitsOptions = EmitsOptions>(emitOptions: E): EmitFn<E>;
export declare function defineEmits<T extends ComponentTypeEmits>(): T extends (...args: any[]) => any ? T : ShortEmits<T>;
export type ComponentTypeEmits = ((...args: any[]) => any) | Record<string, any>;
type RecordToUnion<T extends Record<string, any>> = T[keyof T];
type ShortEmits<T extends Record<string, any>> = UnionToIntersection<RecordToUnion<{
    [K in keyof T]: (evt: K, ...args: T[K]) => void;
}>>;
/**
 * Vue `<script setup>` compiler macro for declaring a component's exposed
 * instance properties when it is accessed by a parent component via template
 * refs.
 *
 * `<script setup>` components are closed by default - i.e. variables inside
 * the `<script setup>` scope is not exposed to parent unless explicitly exposed
 * via `defineExpose`.
 *
 * This is only usable inside `<script setup>`, is compiled away in the
 * output and should **not** be actually called at runtime.
 *
 * @see {@link https://vuejs.org/api/sfc-script-setup.html#defineexpose}
 */
export declare function defineExpose<Exposed extends Record<string, any> = Record<string, any>>(exposed?: Exposed): void;
/**
 * Vue `<script setup>` compiler macro for declaring a component's additional
 * options. This should be used only for options that cannot be expressed via
 * Composition API - e.g. `inheritAttrs`.
 *
 * @see {@link https://vuejs.org/api/sfc-script-setup.html#defineoptions}
 */
export declare function defineOptions<RawBindings = {}, D = {}, C extends ComputedOptions = {}, M extends MethodOptions = {}, Mixin extends ComponentOptionsMixin = ComponentOptionsMixin, Extends extends ComponentOptionsMixin = ComponentOptionsMixin>(options?: ComponentOptionsBase<{}, RawBindings, D, C, M, Mixin, Extends, {}> & {
    /**
     * props should be defined via defineProps().
     */
    props?: never;
    /**
     * emits should be defined via defineEmits().
     */
    emits?: never;
    /**
     * expose should be defined via defineExpose().
     */
    expose?: never;
    /**
     * slots should be defined via defineSlots().
     */
    slots?: never;
}): void;
export declare function defineSlots<S extends Record<string, any> = Record<string, any>>(): StrictUnwrapSlotsType<SlotsType<S>>;
export type ModelRef<T, M extends PropertyKey = string, G = T, S = T> = Ref<G, S> & [
    ModelRef<T, M, G, S>,
    Record<M, true | undefined>
];
export type DefineModelOptions<T = any, G = T, S = T> = {
    get?: (v: T) => G;
    set?: (v: S) => any;
};
/**
 * Vue `<script setup>` compiler macro for declaring a
 * two-way binding prop that can be consumed via `v-model` from the parent
 * component. This will declare a prop with the same name and a corresponding
 * `update:propName` event.
 *
 * If the first argument is a string, it will be used as the prop name;
 * Otherwise the prop name will default to "modelValue". In both cases, you
 * can also pass an additional object which will be used as the prop's options.
 *
 * The returned ref behaves differently depending on whether the parent
 * provided the corresponding v-model props or not:
 * - If yes, the returned ref's value will always be in sync with the parent
 *   prop.
 * - If not, the returned ref will behave like a normal local ref.
 *
 * @example
 * ```ts
 * // default model (consumed via `v-model`)
 * const modelValue = defineModel<string>()
 * modelValue.value = "hello"
 *
 * // default model with options
 * const modelValue = defineModel<string>({ required: true })
 *
 * // with specified name (consumed via `v-model:count`)
 * const count = defineModel<number>('count')
 * count.value++
 *
 * // with specified name and default value
 * const count = defineModel<number>('count', { default: 0 })
 * ```
 */
export declare function defineModel<T, M extends PropertyKey = string, G = T, S = T>(options: ({
    default: any;
} | {
    required: true;
}) & PropOptions<T> & DefineModelOptions<T, G, S>): ModelRef<T, M, G, S>;
export declare function defineModel<T, M extends PropertyKey = string, G = T, S = T>(options?: PropOptions<T> & DefineModelOptions<T, G, S>): ModelRef<T | undefined, M, G | undefined, S | undefined>;
export declare function defineModel<T, M extends PropertyKey = string, G = T, S = T>(name: string, options: ({
    default: any;
} | {
    required: true;
}) & PropOptions<T> & DefineModelOptions<T, G, S>): ModelRef<T, M, G, S>;
export declare function defineModel<T, M extends PropertyKey = string, G = T, S = T>(name: string, options?: PropOptions<T> & DefineModelOptions<T, G, S>): ModelRef<T | undefined, M, G | undefined, S | undefined>;
type NotUndefined<T> = T extends undefined ? never : T;
type MappedOmit<T, K extends keyof any> = {
    [P in keyof T as P extends K ? never : P]: T[P];
};
type InferDefaults<T> = {
    [K in keyof T]?: InferDefault<T, T[K]>;
};
type NativeType = null | undefined | number | string | boolean | symbol | Function;
type InferDefault<P, T> = ((props: P) => T & {}) | (T extends NativeType ? T : never);
type PropsWithDefaults<T, Defaults extends InferDefaults<T>, BKeys extends keyof T> = T extends unknown ? Readonly<MappedOmit<T, keyof Defaults>> & {
    readonly [K in keyof Defaults as K extends keyof T ? K : never]-?: K extends keyof T ? Defaults[K] extends undefined ? IfAny<Defaults[K], NotUndefined<T[K]>, T[K]> : NotUndefined<T[K]> : never;
} & {
    readonly [K in BKeys]-?: K extends keyof Defaults ? Defaults[K] extends undefined ? boolean | undefined : boolean : boolean;
} : never;
/**
 * Vue `<script setup>` compiler macro for providing props default values when
 * using type-based `defineProps` declaration.
 *
 * Example usage:
 * ```ts
 * withDefaults(defineProps<{
 *   size?: number
 *   labels?: string[]
 * }>(), {
 *   size: 3,
 *   labels: () => ['default label']
 * })
 * ```
 *
 * This is only usable inside `<script setup>`, is compiled away in the output
 * and should **not** be actually called at runtime.
 *
 * @see {@link https://vuejs.org/guide/typescript/composition-api.html#typing-component-props}
 */
export declare function withDefaults<T, BKeys extends keyof T, Defaults extends InferDefaults<T>>(props: DefineProps<T, BKeys>, defaults: Defaults): PropsWithDefaults<T, Defaults, BKeys>;
export declare function useSlots(): SetupContext['slots'];
export declare function useAttrs(): SetupContext['attrs'];
export {};
