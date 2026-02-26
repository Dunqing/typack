import { type NodeTransform, type TransformContext } from '../transform';
import { type DirectiveNode, type ElementNode, type ExpressionNode, type ForNode, type ForParseResult } from '../ast';
export declare const transformFor: NodeTransform;
export declare function processFor(node: ElementNode, dir: DirectiveNode, context: TransformContext, processCodegen?: (forNode: ForNode) => (() => void) | undefined): (() => void) | undefined;
export declare function finalizeForParseResult(result: ForParseResult, context: TransformContext): void;
export declare function createForLoopParams({ value, key, index }: ForParseResult, memoArgs?: ExpressionNode[]): ExpressionNode[];
