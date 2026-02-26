import { type CacheExpression, type ComponentNode, ConstantTypes, type PlainElementNode, type RootNode, type SimpleExpressionNode, type TemplateChildNode, type TemplateNode } from '../ast';
import type { TransformContext } from '../transform';
export declare function cacheStatic(root: RootNode, context: TransformContext): void;
export declare function getSingleElementRoot(root: RootNode): PlainElementNode | ComponentNode | TemplateNode | null;
export declare function getConstantType(node: TemplateChildNode | SimpleExpressionNode | CacheExpression, context: TransformContext): ConstantTypes;
