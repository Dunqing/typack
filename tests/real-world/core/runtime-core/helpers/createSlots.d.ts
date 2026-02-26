import type { VNode } from '../vnode';
type SSRSlot = (...args: any[]) => VNode[] | undefined;
interface CompiledSlotDescriptor {
    name: string;
    fn: SSRSlot;
    key?: string;
}
/**
 * Compiler runtime helper for creating dynamic slots object
 * @private
 */
export declare function createSlots(slots: Record<string, SSRSlot>, dynamicSlots: (CompiledSlotDescriptor | CompiledSlotDescriptor[] | undefined)[]): Record<string, SSRSlot>;
export {};
