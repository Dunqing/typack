import { type ComponentOptions, type FunctionalComponent } from '../component';
import type { InternalSlots } from '../componentSlots';
export declare const legacySlotProxyHandlers: ProxyHandler<InternalSlots>;
export declare function convertLegacyFunctionalComponent(comp: ComponentOptions): FunctionalComponent;
