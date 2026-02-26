import { type NodeTransform, type SlotOutletNode } from '@vue/compiler-dom';
import { type SSRTransformContext } from '../ssrCodegenTransform';
export declare const ssrTransformSlotOutlet: NodeTransform;
export declare function ssrProcessSlotOutlet(node: SlotOutletNode, context: SSRTransformContext): void;
