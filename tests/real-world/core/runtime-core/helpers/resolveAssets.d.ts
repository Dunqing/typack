import { type ConcreteComponent } from '../component';
import type { Directive } from '../directives';
import type { VNodeTypes } from '../vnode';
export declare const COMPONENTS = "components";
export declare const DIRECTIVES = "directives";
export declare const FILTERS = "filters";
export type AssetTypes = typeof COMPONENTS | typeof DIRECTIVES | typeof FILTERS;
/**
 * @private
 */
export declare function resolveComponent(name: string, maybeSelfReference?: boolean): ConcreteComponent | string;
export declare const NULL_DYNAMIC_COMPONENT: unique symbol;
/**
 * @private
 */
export declare function resolveDynamicComponent(component: unknown): VNodeTypes;
/**
 * @private
 */
export declare function resolveDirective(name: string): Directive | undefined;
