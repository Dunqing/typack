/**
Runtime helper for applying directives to a vnode. Example usage:

const comp = resolveComponent('comp')
const foo = resolveDirective('foo')
const bar = resolveDirective('bar')

return withDirectives(h(comp), [
  [foo, this.x],
  [bar, this.y]
])
*/
import type { VNode } from './vnode';
import { type ComponentInternalInstance, type Data } from './component';
import type { ComponentPublicInstance } from './componentPublicInstance';
export interface DirectiveBinding<Value = any, Modifiers extends string = string, Arg = any> {
    instance: ComponentPublicInstance | Record<string, any> | null;
    value: Value;
    oldValue: Value | null;
    arg?: Arg;
    modifiers: DirectiveModifiers<Modifiers>;
    dir: ObjectDirective<any, Value, Modifiers, Arg>;
}
export type DirectiveHook<HostElement = any, Prev = VNode<any, HostElement> | null, Value = any, Modifiers extends string = string, Arg = any> = (el: HostElement, binding: DirectiveBinding<Value, Modifiers, Arg>, vnode: VNode<any, HostElement>, prevVNode: Prev) => void;
export type SSRDirectiveHook<Value = any, Modifiers extends string = string, Arg = any> = (binding: DirectiveBinding<Value, Modifiers, Arg>, vnode: VNode) => Data | undefined;
export interface ObjectDirective<HostElement = any, Value = any, Modifiers extends string = string, Arg = any> {
    created?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
    beforeMount?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
    mounted?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
    beforeUpdate?: DirectiveHook<HostElement, VNode<any, HostElement>, Value, Modifiers, Arg>;
    updated?: DirectiveHook<HostElement, VNode<any, HostElement>, Value, Modifiers, Arg>;
    beforeUnmount?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
    unmounted?: DirectiveHook<HostElement, null, Value, Modifiers, Arg>;
    getSSRProps?: SSRDirectiveHook<Value, Modifiers, Arg>;
    deep?: boolean;
}
export type FunctionDirective<HostElement = any, V = any, Modifiers extends string = string, Arg = any> = DirectiveHook<HostElement, any, V, Modifiers, Arg>;
export type Directive<HostElement = any, Value = any, Modifiers extends string = string, Arg = any> = ObjectDirective<HostElement, Value, Modifiers, Arg> | FunctionDirective<HostElement, Value, Modifiers, Arg>;
export type DirectiveModifiers<K extends string = string> = Partial<Record<K, boolean>>;
export declare function validateDirectiveName(name: string): void;
export type DirectiveArguments = Array<[Directive | undefined] | [Directive | undefined, any] | [Directive | undefined, any, any] | [Directive | undefined, any, any, DirectiveModifiers]>;
/**
 * Adds directives to a VNode.
 */
export declare function withDirectives<T extends VNode>(vnode: T, directives: DirectiveArguments): T;
export declare function invokeDirectiveHook(vnode: VNode, prevVNode: VNode | null, instance: ComponentInternalInstance | null, name: keyof ObjectDirective): void;
