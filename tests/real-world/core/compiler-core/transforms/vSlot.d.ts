import { type DirectiveNode, type ElementNode, type ExpressionNode, type FunctionExpression, type SlotsExpression, type SourceLocation, type TemplateChildNode } from '../ast';
import type { NodeTransform, TransformContext } from '../transform';
export declare const trackSlotScopes: NodeTransform;
export declare const trackVForSlotScopes: NodeTransform;
export type SlotFnBuilder = (slotProps: ExpressionNode | undefined, vFor: DirectiveNode | undefined, slotChildren: TemplateChildNode[], loc: SourceLocation) => FunctionExpression;
export declare function buildSlots(node: ElementNode, context: TransformContext, buildSlotFn?: SlotFnBuilder): {
    slots: SlotsExpression;
    hasDynamicSlots: boolean;
};
