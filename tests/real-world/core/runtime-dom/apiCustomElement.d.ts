import { type App, type Component, type ComponentCustomElementInterface, type ComponentInjectOptions, type ComponentObjectPropsOptions, type ComponentOptions, type ComponentOptionsBase, type ComponentOptionsMixin, type ComponentProvideOptions, type ComponentPublicInstance, type ComputedOptions, type ConcreteComponent, type CreateAppFunction, type CreateComponentPublicInstanceWithMixins, type DefineComponent, type Directive, type EmitsOptions, type EmitsToProps, type ExtractPropTypes, type MethodOptions, type RenderFunction, type SetupContext, type SlotsType } from '@vue/runtime-core';
export type VueElementConstructor<P = {}> = {
    new (initialProps?: Record<string, any>): VueElement & P;
};
export interface CustomElementOptions {
    styles?: string[];
    shadowRoot?: boolean;
    shadowRootOptions?: Omit<ShadowRootInit, 'mode'>;
    nonce?: string;
    configureApp?: (app: App) => void;
}
export declare function defineCustomElement<Props, RawBindings = object>(setup: (props: Props, ctx: SetupContext) => RawBindings | RenderFunction, options?: Pick<ComponentOptions, 'name' | 'inheritAttrs' | 'emits'> & CustomElementOptions & {
    props?: (keyof Props)[];
}): VueElementConstructor<Props>;
export declare function defineCustomElement<Props, RawBindings = object>(setup: (props: Props, ctx: SetupContext) => RawBindings | RenderFunction, options?: Pick<ComponentOptions, 'name' | 'inheritAttrs' | 'emits'> & CustomElementOptions & {
    props?: ComponentObjectPropsOptions<Props>;
}): VueElementConstructor<Props>;
export declare function defineCustomElement<RuntimePropsOptions extends ComponentObjectPropsOptions = ComponentObjectPropsOptions, PropsKeys extends string = string, RuntimeEmitsOptions extends EmitsOptions = {}, EmitsKeys extends string = string, Data = {}, SetupBindings = {}, Computed extends ComputedOptions = {}, Methods extends MethodOptions = {}, Mixin extends ComponentOptionsMixin = ComponentOptionsMixin, Extends extends ComponentOptionsMixin = ComponentOptionsMixin, InjectOptions extends ComponentInjectOptions = {}, InjectKeys extends string = string, Slots extends SlotsType = {}, LocalComponents extends Record<string, Component> = {}, Directives extends Record<string, Directive> = {}, Exposed extends string = string, Provide extends ComponentProvideOptions = ComponentProvideOptions, InferredProps = string extends PropsKeys ? ComponentObjectPropsOptions extends RuntimePropsOptions ? {} : ExtractPropTypes<RuntimePropsOptions> : {
    [key in PropsKeys]?: any;
}, ResolvedProps = InferredProps & EmitsToProps<RuntimeEmitsOptions>>(options: CustomElementOptions & {
    props?: (RuntimePropsOptions & ThisType<void>) | PropsKeys[];
} & ComponentOptionsBase<ResolvedProps, SetupBindings, Data, Computed, Methods, Mixin, Extends, RuntimeEmitsOptions, EmitsKeys, {}, // Defaults
InjectOptions, InjectKeys, Slots, LocalComponents, Directives, Exposed, Provide> & ThisType<CreateComponentPublicInstanceWithMixins<Readonly<ResolvedProps>, SetupBindings, Data, Computed, Methods, Mixin, Extends, RuntimeEmitsOptions, EmitsKeys, {}, false, InjectOptions, Slots, LocalComponents, Directives, Exposed>>, extraOptions?: CustomElementOptions): VueElementConstructor<ResolvedProps>;
export declare function defineCustomElement<T extends {
    new (...args: any[]): ComponentPublicInstance<any>;
}>(options: T, extraOptions?: CustomElementOptions): VueElementConstructor<T extends DefineComponent<infer P, any, any, any> ? P : unknown>;
export declare const defineSSRCustomElement: typeof defineCustomElement;
declare const BaseClass: typeof HTMLElement;
type InnerComponentDef = ConcreteComponent & CustomElementOptions;
export declare class VueElement extends BaseClass implements ComponentCustomElementInterface {
    /**
     * Component def - note this may be an AsyncWrapper, and this._def will
     * be overwritten by the inner component when resolved.
     */
    private _def;
    private _props;
    private _createApp;
    _isVueCE: boolean;
    private _connected;
    private _resolved;
    private _patching;
    private _dirty;
    private _numberProps;
    private _styleChildren;
    private _pendingResolve;
    private _parent;
    /**
     * dev only
     */
    private _styles?;
    /**
     * dev only
     */
    private _childStyles?;
    private _ob?;
    private _slots?;
    constructor(
    /**
     * Component def - note this may be an AsyncWrapper, and this._def will
     * be overwritten by the inner component when resolved.
     */
    _def: InnerComponentDef, _props?: Record<string, any>, _createApp?: CreateAppFunction<Element>);
    connectedCallback(): void;
    private _setParent;
    private _inheritParentContext;
    disconnectedCallback(): void;
    private _processMutations;
    /**
     * resolve inner component definition (handle possible async component)
     */
    private _resolveDef;
    private _mount;
    private _resolveProps;
    protected _setAttr(key: string): void;
    private _update;
    private _createVNode;
    private _applyStyles;
    /**
     * Only called when shadowRoot is false
     */
    private _parseSlots;
    /**
     * Only called when shadowRoot is false
     */
    private _renderSlots;
}
export declare function useHost(caller?: string): VueElement | null;
/**
 * Retrieve the shadowRoot of the current custom element. Only usable in setup()
 * of a `defineCustomElement` component.
 */
export declare function useShadowRoot(): ShadowRoot | null;
export {};
