import type { NodeTransform, TransformContext } from '../transform';
import { type ExpressionNode, type SlotOutletNode } from '../ast';
import { type PropsExpression } from './transformElement';
export declare const transformSlotOutlet: NodeTransform;
interface SlotOutletProcessResult {
    slotName: string | ExpressionNode;
    slotProps: PropsExpression | undefined;
}
export declare function processSlotOutlet(node: SlotOutletNode, context: TransformContext): SlotOutletProcessResult;
export {};
