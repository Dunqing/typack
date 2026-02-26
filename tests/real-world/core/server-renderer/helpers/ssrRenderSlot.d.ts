import { type ComponentInternalInstance, type Slots } from 'vue';
import { type Props, type PushFn } from '../render';
export type SSRSlots = Record<string, SSRSlot>;
export type SSRSlot = (props: Props, push: PushFn, parentComponent: ComponentInternalInstance | null, scopeId: string | null) => void;
export declare function ssrRenderSlot(slots: Slots | SSRSlots, slotName: string, slotProps: Props, fallbackRenderFn: (() => void) | null, push: PushFn, parentComponent: ComponentInternalInstance, slotScopeId?: string): void;
export declare function ssrRenderSlotInner(slots: Slots | SSRSlots, slotName: string, slotProps: Props, fallbackRenderFn: (() => void) | null, push: PushFn, parentComponent: ComponentInternalInstance, slotScopeId?: string, transition?: boolean): void;
