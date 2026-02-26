import { type Component, type ComponentInternalInstance, type Slots } from 'vue';
import { type Props, type SSRBuffer } from '../render';
import type { SSRSlots } from './ssrRenderSlot';
export declare function ssrRenderComponent(comp: Component, props?: Props | null, children?: Slots | SSRSlots | null, parentComponent?: ComponentInternalInstance | null, slotScopeId?: string): SSRBuffer | Promise<SSRBuffer>;
