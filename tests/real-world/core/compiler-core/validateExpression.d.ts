import type { SimpleExpressionNode } from './ast';
import type { TransformContext } from './transform';
/**
 * Validate a non-prefixed expression.
 * This is only called when using the in-browser runtime compiler since it
 * doesn't prefix expressions.
 */
export declare function validateBrowserExpression(node: SimpleExpressionNode, context: TransformContext, asParams?: boolean, asRawStatements?: boolean): void;
