import type { Data } from '../component';
import type { Slots } from '../componentSlots';
import { type VNode, type VNodeArrayChildren } from '../vnode';
/**
 * Compiler runtime helper for rendering `<slot/>`
 * @private
 */
export declare function renderSlot(slots: Slots, name: string, props?: Data, fallback?: () => VNodeArrayChildren, noSlotted?: boolean): VNode;
export declare function ensureValidVNode(vnodes: VNodeArrayChildren): VNodeArrayChildren | null;
